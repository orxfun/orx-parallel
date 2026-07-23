use crate::NumThreads;
use crate::pool::ParThreadPool;
use crate::pool::env::max_num_threads_by_env_and_resource;
use alloc::format;
use core::num::NonZeroUsize;
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use js_sys::Promise;
use std::any::Any;
use std::boxed::Box;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

const WASM_WEB3_THREAD_POOL_UNINITIALIZED: u8 = 0;
const WASM_WEB3_THREAD_POOL_INITIALIZED: u8 = 1;

#[cfg_attr(not(target_feature = "atomics"), allow(dead_code))]
static WASM_WEB3_THREAD_POOL_STATE: AtomicU8 = AtomicU8::new(WASM_WEB3_THREAD_POOL_UNINITIALIZED);
#[cfg_attr(not(target_feature = "atomics"), allow(dead_code))]
static WASM_WEB3_THREAD_POOL_NUM_THREADS: AtomicUsize = AtomicUsize::new(0);
static WASM_WEB3_RUNTIME: OnceLock<Arc<Inner>> = OnceLock::new();

#[wasm_bindgen(module = "/src/pool/pool_impl/wasm_web_start_workers.js")]
extern "C" {
    #[wasm_bindgen(js_name = startWorkers)]
    fn start_workers(module: JsValue, memory: JsValue, num_threads: usize) -> Promise;
}

struct Inner {
    shared: Arc<WorkerShared>,
    spawned_workers: usize,
}

struct WorkerShared {
    state: Mutex<WorkerState>,
    cv: Condvar,
}

struct WorkerState {
    shutdown: bool,
    active_scope_addr: Option<usize>,
    queue: VecDeque<Task>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        {
            let mut state = self.shared.state.lock().expect("poisoned pool lock");
            state.shutdown = true;
            while let Some(task) = state.queue.pop_front() {
                // A scope waits for all submitted work; this is a defensive fallback.
                unsafe { task.drop() };
            }
        }
        self.shared.cv.notify_all();
    }
}

struct ScopeRuntime {
    pending: AtomicUsize,
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    completion_lock: Mutex<()>,
    completion_cv: Condvar,
    panic: Mutex<Option<Box<dyn Any + Send>>>,
}

impl ScopeRuntime {
    fn new() -> Self {
        Self {
            pending: AtomicUsize::new(0),
            completion_lock: Mutex::new(()),
            completion_cv: Condvar::new(),
            panic: Mutex::new(None),
        }
    }

    fn begin_task(&self) {
        self.pending.fetch_add(1, Ordering::AcqRel);
    }

    fn complete_task(&self) {
        if self.pending.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.completion_cv.notify_all();
        }
    }

    fn wait_for_completion(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            while self.pending.load(Ordering::Acquire) != 0 {
                core::hint::spin_loop();
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut guard = self
                .completion_lock
                .lock()
                .expect("poisoned scope completion lock");
            while self.pending.load(Ordering::Acquire) != 0 {
                guard = self
                    .completion_cv
                    .wait(guard)
                    .expect("poisoned scope completion lock");
            }
        }
    }

    fn record_panic(&self, err: Box<dyn Any + Send>) {
        let mut panic_slot = self.panic.lock().expect("poisoned scope panic lock");
        if panic_slot.is_none() {
            *panic_slot = Some(err);
        }
    }

    fn take_panic(&self) -> Option<Box<dyn Any + Send>> {
        self.panic.lock().expect("poisoned scope panic lock").take()
    }
}

pub struct ScopeRef<'env> {
    shared: *const WorkerShared,
    runtime: *const ScopeRuntime,
    inline_only: bool,
    _marker: PhantomData<&'env ()>,
}

impl<'env> ScopeRef<'env> {
    fn shared(&self) -> &WorkerShared {
        unsafe { &*self.shared }
    }

    fn runtime(&self) -> &ScopeRuntime {
        unsafe { &*self.runtime }
    }
}

#[cfg_attr(not(target_feature = "atomics"), allow(dead_code))]
struct Task {
    data: *mut (),
    run_fn: unsafe fn(*mut ()),
    drop_fn: unsafe fn(*mut ()),
}

unsafe impl Send for Task {}

#[cfg_attr(not(target_feature = "atomics"), allow(dead_code))]
impl Task {
    fn new<W>(work: W) -> Self
    where
        W: Fn() + Send,
    {
        unsafe fn run_impl<W>(data: *mut ())
        where
            W: Fn() + Send,
        {
            let work = unsafe { Box::from_raw(data as *mut W) };
            (*work)();
        }

        unsafe fn drop_impl<W>(data: *mut ())
        where
            W: Fn() + Send,
        {
            drop(unsafe { Box::from_raw(data as *mut W) });
        }

        let boxed = Box::new(work);
        Self {
            data: Box::into_raw(boxed) as *mut (),
            run_fn: run_impl::<W>,
            drop_fn: drop_impl::<W>,
        }
    }

    unsafe fn run(self) {
        unsafe { (self.run_fn)(self.data) };
    }

    unsafe fn drop(self) {
        unsafe { (self.drop_fn)(self.data) };
    }
}

#[cfg_attr(not(target_feature = "atomics"), allow(dead_code))]
fn worker_loop(shared: Arc<WorkerShared>) {
    loop {
        let (task, runtime_ptr) = {
            let mut state = shared.state.lock().expect("poisoned pool lock");
            loop {
                if state.shutdown {
                    return;
                }

                if let Some(task) = state.queue.pop_front() {
                    let runtime_ptr = state
                        .active_scope_addr
                        .expect("active scope must be set while queue is non-empty");
                    break (task, runtime_ptr as *const ScopeRuntime);
                }

                state = shared.cv.wait(state).expect("poisoned pool lock");
            }
        };

        let runtime = unsafe { &*runtime_ptr };
        let result = catch_unwind(AssertUnwindSafe(|| unsafe { task.run() }));
        if let Err(err) = result {
            runtime.record_panic(err);
        }
        runtime.complete_task();
    }
}

#[cfg_attr(not(target_feature = "atomics"), allow(dead_code))]
fn init_runtime(num_threads: NonZeroUsize) -> Arc<Inner> {
    let shared = Arc::new(WorkerShared {
        state: Mutex::new(WorkerState {
            shutdown: false,
            active_scope_addr: None,
            queue: VecDeque::new(),
        }),
        cv: Condvar::new(),
    });

    Arc::new(Inner {
        shared,
        spawned_workers: num_threads.get(),
    })
}

/// Initializes the worker-backed wasm thread runtime for `WasmWebPool`.
#[cfg(target_feature = "atomics")]
pub fn init_thread_pool(num_threads: usize) -> js_sys::Promise {
    let num_threads = NonZeroUsize::new(num_threads.max(1)).expect(">0");

    match WASM_WEB3_THREAD_POOL_STATE.compare_exchange(
        WASM_WEB3_THREAD_POOL_UNINITIALIZED,
        WASM_WEB3_THREAD_POOL_INITIALIZED,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(_) => {
            WASM_WEB3_THREAD_POOL_NUM_THREADS.store(num_threads.get(), Ordering::SeqCst);

            let _ = WASM_WEB3_RUNTIME.get_or_init(|| init_runtime(num_threads));

            start_workers(
                wasm_bindgen::module(),
                wasm_bindgen::memory(),
                num_threads.get(),
            )
        }
        Err(WASM_WEB3_THREAD_POOL_INITIALIZED) => {
            let configured_threads = WASM_WEB3_THREAD_POOL_NUM_THREADS.load(Ordering::SeqCst);

            match configured_threads == num_threads.get() {
                true => js_sys::Promise::resolve(&wasm_bindgen::JsValue::UNDEFINED),
                false => js_sys::Promise::reject(&wasm_bindgen::JsValue::from_str(&format!(
                    "init_thread_pool was already called with {configured_threads} threads; refusing to reinitialize with {} threads",
                    num_threads.get()
                ))),
            }
        }
        Err(_) => unreachable!("invalid wasm-web-threads init state"),
    }
}

#[cfg(target_feature = "atomics")]
/// Returns `(configured_threads, spawned_workers)` for the active wasm-web-threads runtime.
pub fn wasm_web_runtime_info() -> (usize, usize) {
    let configured_threads = WASM_WEB3_THREAD_POOL_NUM_THREADS.load(Ordering::SeqCst);
    let spawned_workers = runtime().spawned_workers;
    (configured_threads, spawned_workers)
}

#[cfg(target_feature = "atomics")]
#[wasm_bindgen]
/// Worker entrypoint called from the wasm worker helper after wasm init.
pub fn wasm_web_start_worker() {
    let shared = Arc::clone(&runtime().shared);
    worker_loop(shared);
}

fn assert_wasm_thread_pool_initialized() {
    assert!(
        cfg!(target_feature = "atomics"),
        "Wasm web threading requires atomics-enabled wasm build flags; see docs/wasm-plan-b.md."
    );

    assert_eq!(
        WASM_WEB3_THREAD_POOL_STATE.load(Ordering::SeqCst),
        WASM_WEB3_THREAD_POOL_INITIALIZED,
        "Wasm web thread pool is not initialized. Call and await init_thread_pool(...) before running parallel computations."
    )
}

fn runtime() -> &'static Arc<Inner> {
    assert_wasm_thread_pool_initialized();
    WASM_WEB3_RUNTIME.get_or_init(|| {
        let num_threads = WASM_WEB3_THREAD_POOL_NUM_THREADS.load(Ordering::SeqCst);
        let num_threads = NonZeroUsize::new(num_threads)
            .expect("wasm web configured thread count must be > 0 after init_thread_pool");
        init_runtime(num_threads)
    })
}

/// wasm web-thread pool adapter for the simplified wasm backend.
#[derive(Clone, Copy, Debug)]
pub struct WasmWebPool {
    max_num_threads: NonZeroUsize,
}

impl Default for WasmWebPool {
    fn default() -> Self {
        Self::new(NumThreads::Auto)
    }
}

impl WasmWebPool {
    /// Creates a new wasm web-thread pool adapter.
    pub fn new(num_threads: impl Into<NumThreads>) -> Self {
        let max_num_threads = match num_threads.into() {
            NumThreads::Auto => max_num_threads_by_env_and_resource(),
            NumThreads::Max(n) => max_num_threads_by_env_and_resource().min(n),
        };

        Self { max_num_threads }
    }

    fn scoped_computation_impl<'env, 'scope, F>(&'env self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s ScopeRef<'env>) + Send,
    {
        let scope_runtime = ScopeRuntime::new();

        {
            let runtime_ref = runtime();
            let mut state = runtime_ref.shared.state.lock().expect("poisoned pool lock");
            debug_assert!(state.active_scope_addr.is_none());
            state.active_scope_addr = Some(&scope_runtime as *const ScopeRuntime as usize);
        }

        let scope_ref = ScopeRef {
            shared: Arc::as_ptr(&runtime().shared),
            runtime: &scope_runtime,
            inline_only: runtime().spawned_workers == 0,
            _marker: PhantomData,
        };

        let user_result = catch_unwind(AssertUnwindSafe(|| f(&scope_ref)));

        scope_runtime.wait_for_completion();

        {
            let runtime_ref = runtime();
            let mut state = runtime_ref.shared.state.lock().expect("poisoned pool lock");
            state.active_scope_addr = None;
            debug_assert!(state.queue.is_empty());
        }

        if let Err(err) = user_result {
            resume_unwind(err);
        }

        if let Some(err) = scope_runtime.take_panic() {
            resume_unwind(err);
        }
    }
}

impl ParThreadPool for WasmWebPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s ScopeRef<'env>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.runtime().begin_task();

        if s.inline_only {
            let result = catch_unwind(AssertUnwindSafe(work));
            if let Err(err) = result {
                s.runtime().record_panic(err);
            }
            s.runtime().complete_task();
            return;
        }

        let task = Task::new(work);

        {
            let mut state = s.shared().state.lock().expect("poisoned pool lock");
            state.queue.push_back(task);
        }

        s.shared().cv.notify_one();
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s ScopeRef<'env>) + Send,
    {
        self.scoped_computation_impl(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }
}

impl ParThreadPool for &WasmWebPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s ScopeRef<'env>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        <WasmWebPool as ParThreadPool>::run_in_scope(s, work)
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s ScopeRef<'env>) + Send,
    {
        (*self).scoped_computation_impl(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }
}
