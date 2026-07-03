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
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock};
use std::vec::Vec;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

const WASM_WEB2_THREAD_POOL_UNINITIALIZED: u8 = 0;
const WASM_WEB2_THREAD_POOL_INITIALIZED: u8 = 1;

#[cfg_attr(not(target_feature = "atomics"), allow(dead_code))]
static WASM_WEB2_THREAD_POOL_STATE: AtomicU8 = AtomicU8::new(WASM_WEB2_THREAD_POOL_UNINITIALIZED);
#[cfg_attr(not(target_feature = "atomics"), allow(dead_code))]
static WASM_WEB2_THREAD_POOL_NUM_THREADS: AtomicUsize = AtomicUsize::new(0);
static WASM_WEB2_RUNTIME: OnceLock<Arc<Inner>> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
const MAIN_ASSIST_GRACE_SPINS: usize = 256;
#[cfg(target_arch = "wasm32")]
const MAIN_ASSIST_STALL_SPINS: usize = 4096;
#[cfg(target_arch = "wasm32")]
const MAIN_ASSIST_BACKLOG_THRESHOLD: usize = 2;
#[cfg(target_arch = "wasm32")]
const MAIN_ASSIST_FAILED_COOLDOWN_SPINS: usize = 64;
#[cfg(target_arch = "wasm32")]
const MAIN_ASSIST_WATCHDOG_SPINS: usize = 4096;

#[wasm_bindgen(module = "/src/pool/pool_impl/worker_helpers2.js")]
extern "C" {
    #[wasm_bindgen(js_name = startWorkers)]
    fn start_workers(module: JsValue, memory: JsValue, num_threads: usize) -> Promise;
}

struct Inner {
    shared: Arc<WorkerShared>,
    spawned_workers: usize,
    telemetry: RuntimeTelemetry,
}

struct RuntimeTelemetry {
    tasks_enqueued: AtomicUsize,
    tasks_run_by_workers: AtomicUsize,
    tasks_run_by_main: AtomicUsize,
    notify_count: AtomicUsize,
    queue_depth_high_water: AtomicUsize,
    queue_pop_count: AtomicUsize,
    queue_empty_poll_count: AtomicUsize,
    main_assist_time_ns: AtomicUsize,
    state_try_lock_fail_count: AtomicUsize,
    state_try_lock_spin_iters: AtomicUsize,
    completion_notify_count: AtomicUsize,
    main_assist_attempt_count: AtomicUsize,
    main_assist_success_count: AtomicUsize,
    worker_id_alloc: AtomicUsize,
    worker_runs_by_id: Mutex<Vec<usize>>,
}

pub struct RuntimeTelemetryExtendedSnapshot {
    pub queue_pop_count: usize,
    pub queue_empty_poll_count: usize,
    pub main_assist_time_ns: usize,
    pub state_try_lock_fail_count: usize,
    pub state_try_lock_spin_iters: usize,
    pub completion_notify_count: usize,
    pub main_assist_attempt_count: usize,
    pub main_assist_success_count: usize,
    pub worker_runs_by_id: alloc::vec::Vec<usize>,
}

impl RuntimeTelemetry {
    fn new() -> Self {
        Self {
            tasks_enqueued: AtomicUsize::new(0),
            tasks_run_by_workers: AtomicUsize::new(0),
            tasks_run_by_main: AtomicUsize::new(0),
            notify_count: AtomicUsize::new(0),
            queue_depth_high_water: AtomicUsize::new(0),
            queue_pop_count: AtomicUsize::new(0),
            queue_empty_poll_count: AtomicUsize::new(0),
            main_assist_time_ns: AtomicUsize::new(0),
            state_try_lock_fail_count: AtomicUsize::new(0),
            state_try_lock_spin_iters: AtomicUsize::new(0),
            completion_notify_count: AtomicUsize::new(0),
            main_assist_attempt_count: AtomicUsize::new(0),
            main_assist_success_count: AtomicUsize::new(0),
            worker_id_alloc: AtomicUsize::new(0),
            worker_runs_by_id: Mutex::new(Vec::new()),
        }
    }

    fn reset(&self) {
        self.tasks_enqueued.store(0, Ordering::SeqCst);
        self.tasks_run_by_workers.store(0, Ordering::SeqCst);
        self.tasks_run_by_main.store(0, Ordering::SeqCst);
        self.notify_count.store(0, Ordering::SeqCst);
        self.queue_depth_high_water.store(0, Ordering::SeqCst);
        self.queue_pop_count.store(0, Ordering::SeqCst);
        self.queue_empty_poll_count.store(0, Ordering::SeqCst);
        self.main_assist_time_ns.store(0, Ordering::SeqCst);
        self.state_try_lock_fail_count.store(0, Ordering::SeqCst);
        self.state_try_lock_spin_iters.store(0, Ordering::SeqCst);
        self.completion_notify_count.store(0, Ordering::SeqCst);
        self.main_assist_attempt_count.store(0, Ordering::SeqCst);
        self.main_assist_success_count.store(0, Ordering::SeqCst);
        self.worker_id_alloc.store(0, Ordering::SeqCst);
        self.worker_runs_by_id
            .lock()
            .expect("poisoned worker_runs_by_id lock")
            .clear();
    }

    fn snapshot(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.tasks_enqueued.load(Ordering::SeqCst),
            self.tasks_run_by_workers.load(Ordering::SeqCst),
            self.tasks_run_by_main.load(Ordering::SeqCst),
            self.notify_count.load(Ordering::SeqCst),
            self.queue_depth_high_water.load(Ordering::SeqCst),
        )
    }

    fn snapshot_extended(&self) -> RuntimeTelemetryExtendedSnapshot {
        RuntimeTelemetryExtendedSnapshot {
            queue_pop_count: self.queue_pop_count.load(Ordering::SeqCst),
            queue_empty_poll_count: self.queue_empty_poll_count.load(Ordering::SeqCst),
            main_assist_time_ns: self.main_assist_time_ns.load(Ordering::SeqCst),
            state_try_lock_fail_count: self.state_try_lock_fail_count.load(Ordering::SeqCst),
            state_try_lock_spin_iters: self.state_try_lock_spin_iters.load(Ordering::SeqCst),
            completion_notify_count: self.completion_notify_count.load(Ordering::SeqCst),
            main_assist_attempt_count: self.main_assist_attempt_count.load(Ordering::SeqCst),
            main_assist_success_count: self.main_assist_success_count.load(Ordering::SeqCst),
            worker_runs_by_id: self
                .worker_runs_by_id
                .lock()
                .expect("poisoned worker_runs_by_id lock")
                .clone(),
        }
    }

    fn register_worker(&self) -> usize {
        let worker_id = self.worker_id_alloc.fetch_add(1, Ordering::Relaxed);
        let mut worker_runs = self
            .worker_runs_by_id
            .lock()
            .expect("poisoned worker_runs_by_id lock");
        if worker_runs.len() <= worker_id {
            worker_runs.resize(worker_id + 1, 0);
        }
        worker_id
    }

    fn record_worker_run(&self, worker_id: usize) {
        let mut worker_runs = self
            .worker_runs_by_id
            .lock()
            .expect("poisoned worker_runs_by_id lock");
        if worker_runs.len() <= worker_id {
            worker_runs.resize(worker_id + 1, 0);
        }
        worker_runs[worker_id] = worker_runs[worker_id].saturating_add(1);
    }
}

fn update_high_water(mark: &AtomicUsize, value: usize) {
    let mut current = mark.load(Ordering::Relaxed);
    while value > current {
        match mark.compare_exchange_weak(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(next) => current = next,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn now_ns() -> usize {
    (js_sys::Date::now() * 1_000_000.0) as usize
}

fn lock_pool_state<'a>(shared: &'a WorkerShared) -> MutexGuard<'a, WorkerState> {
    #[cfg(target_arch = "wasm32")]
    {
        loop {
            match shared.state.try_lock() {
                Ok(guard) => return guard,
                Err(_) => {
                    if let Some(runtime) = WASM_WEB2_RUNTIME.get() {
                        runtime
                            .telemetry
                            .state_try_lock_fail_count
                            .fetch_add(1, Ordering::Relaxed);
                        runtime
                            .telemetry
                            .state_try_lock_spin_iters
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    core::hint::spin_loop()
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        shared.state.lock().expect("poisoned pool lock")
    }
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
            let mut state = lock_pool_state(&self.shared);
            state.shutdown = true;
            while let Some(task) = state.queue.pop_front() {
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

#[cfg_attr(not(target_feature = "atomics"), allow(dead_code))]
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
            if let Some(runtime) = WASM_WEB2_RUNTIME.get() {
                runtime
                    .telemetry
                    .completion_notify_count
                    .fetch_add(1, Ordering::Relaxed);
            }
            self.completion_cv.notify_all();
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
    let worker_id = WASM_WEB2_RUNTIME
        .get()
        .map(|runtime| runtime.telemetry.register_worker())
        .unwrap_or(0);

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
                    if let Some(runtime) = WASM_WEB2_RUNTIME.get() {
                        runtime
                            .telemetry
                            .queue_pop_count
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    break (task, runtime_ptr as *const ScopeRuntime);
                }

                if let Some(runtime) = WASM_WEB2_RUNTIME.get() {
                    runtime
                        .telemetry
                        .queue_empty_poll_count
                        .fetch_add(1, Ordering::Relaxed);
                }

                state = shared.cv.wait(state).expect("poisoned pool lock");
            }
        };

        let scope_runtime = unsafe { &*runtime_ptr };
        let result = catch_unwind(AssertUnwindSafe(|| unsafe { task.run() }));
        if let Some(runtime) = WASM_WEB2_RUNTIME.get() {
            runtime
                .telemetry
                .tasks_run_by_workers
                .fetch_add(1, Ordering::Relaxed);
            runtime.telemetry.record_worker_run(worker_id);
        }
        if let Err(err) = result {
            scope_runtime.record_panic(err);
        }
        scope_runtime.complete_task();
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
        telemetry: RuntimeTelemetry::new(),
    })
}

/// Initializes the worker-backed wasm thread runtime for `WasmWebPool2`.
///
/// This establishes the runtime init contract for the new wasm backend.
#[cfg(target_feature = "atomics")]
pub fn init_thread_pool(num_threads: usize) -> js_sys::Promise {
    let num_threads = NonZeroUsize::new(num_threads.max(1)).expect(">0");

    match WASM_WEB2_THREAD_POOL_STATE.compare_exchange(
        WASM_WEB2_THREAD_POOL_UNINITIALIZED,
        WASM_WEB2_THREAD_POOL_INITIALIZED,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(_) => {
            WASM_WEB2_THREAD_POOL_NUM_THREADS.store(num_threads.get(), Ordering::SeqCst);

            let _ = WASM_WEB2_RUNTIME.get_or_init(|| init_runtime(num_threads));

            start_workers(
                wasm_bindgen::module(),
                wasm_bindgen::memory(),
                num_threads.get(),
            )
        }
        Err(WASM_WEB2_THREAD_POOL_INITIALIZED) => {
            let configured_threads = WASM_WEB2_THREAD_POOL_NUM_THREADS.load(Ordering::SeqCst);

            match configured_threads == num_threads.get() {
                true => js_sys::Promise::resolve(&wasm_bindgen::JsValue::UNDEFINED),
                false => js_sys::Promise::reject(&wasm_bindgen::JsValue::from_str(&format!(
                    "init_thread_pool was already called with {configured_threads} threads; refusing to reinitialize with {} threads",
                    num_threads.get()
                ))),
            }
        }
        Err(_) => unreachable!("invalid wasm-web-threads2 init state"),
    }
}

#[cfg(target_feature = "atomics")]
/// Returns `(configured_threads, spawned_workers)` for the active wasm-web-threads2 runtime.
pub fn wasm_web2_runtime_info() -> (usize, usize) {
    let configured_threads = WASM_WEB2_THREAD_POOL_NUM_THREADS.load(Ordering::SeqCst);
    let spawned_workers = runtime().spawned_workers;
    (configured_threads, spawned_workers)
}

#[cfg(target_feature = "atomics")]
/// Resets runtime performance counters for wasm-web-threads2.
pub fn wasm_web2_perf_reset() {
    if let Some(runtime) = WASM_WEB2_RUNTIME.get() {
        runtime.telemetry.reset();
    }
}

#[cfg(target_feature = "atomics")]
/// Returns performance counters as:
/// `(tasks_enqueued, tasks_run_by_workers, tasks_run_by_main, notify_count, queue_depth_high_water)`.
pub fn wasm_web2_perf_snapshot() -> (usize, usize, usize, usize, usize) {
    if let Some(runtime) = WASM_WEB2_RUNTIME.get() {
        runtime.telemetry.snapshot()
    } else {
        (0, 0, 0, 0, 0)
    }
}

#[cfg(target_feature = "atomics")]
/// Returns extended performance counters for diagnostics.
pub fn wasm_web2_perf_snapshot_extended() -> RuntimeTelemetryExtendedSnapshot {
    if let Some(runtime) = WASM_WEB2_RUNTIME.get() {
        runtime.telemetry.snapshot_extended()
    } else {
        RuntimeTelemetryExtendedSnapshot {
            queue_pop_count: 0,
            queue_empty_poll_count: 0,
            main_assist_time_ns: 0,
            state_try_lock_fail_count: 0,
            state_try_lock_spin_iters: 0,
            completion_notify_count: 0,
            main_assist_attempt_count: 0,
            main_assist_success_count: 0,
            worker_runs_by_id: alloc::vec::Vec::new(),
        }
    }
}

#[cfg(target_feature = "atomics")]
#[wasm_bindgen]
/// Worker entrypoint called from `worker_helpers2.js` after wasm init.
pub fn wasm_web2_start_worker() {
    let shared = Arc::clone(&runtime().shared);
    worker_loop(shared);
}

fn assert_wasm_thread_pool_initialized() {
    assert!(
        cfg!(target_feature = "atomics"),
        "Wasm web threading requires atomics-enabled wasm build flags; see docs/wasm-plan-b.md."
    );

    assert_eq!(
        WASM_WEB2_THREAD_POOL_STATE.load(Ordering::SeqCst),
        WASM_WEB2_THREAD_POOL_INITIALIZED,
        "Wasm web2 thread pool is not initialized. Call and await init_thread_pool(...) before running parallel computations."
    )
}

fn runtime() -> &'static Arc<Inner> {
    assert_wasm_thread_pool_initialized();
    WASM_WEB2_RUNTIME.get_or_init(|| {
        let num_threads = WASM_WEB2_THREAD_POOL_NUM_THREADS.load(Ordering::SeqCst);
        let num_threads = NonZeroUsize::new(num_threads)
            .expect("wasm web2 configured thread count must be > 0 after init_thread_pool");
        init_runtime(num_threads)
    })
}

/// wasm web-thread pool adapter for the new internal backend.
#[derive(Clone, Copy, Debug)]
pub struct WasmWebPool2 {
    max_num_threads: NonZeroUsize,
}

impl Default for WasmWebPool2 {
    fn default() -> Self {
        Self::new(NumThreads::Auto)
    }
}

impl WasmWebPool2 {
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
            let mut state = lock_pool_state(&runtime_ref.shared);
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

        #[cfg(target_arch = "wasm32")]
        {
            for _ in 0..MAIN_ASSIST_GRACE_SPINS {
                if scope_runtime.pending.load(Ordering::Acquire) == 0 {
                    break;
                }
                core::hint::spin_loop();
            }

            let mut last_pending = scope_runtime.pending.load(Ordering::Acquire);
            let mut last_worker_runs = runtime()
                .telemetry
                .tasks_run_by_workers
                .load(Ordering::Relaxed);
            let worker_runs_at_scope_start = last_worker_runs;
            let mut stall_spins = 0usize;
            let mut failed_assist_cooldown = 0usize;
            let mut watchdog_spins = 0usize;

            // Main-thread wasm cannot block on Condvar/Atomics.wait.
            while scope_runtime.pending.load(Ordering::Acquire) != 0 {
                let pending_now = scope_runtime.pending.load(Ordering::Acquire);
                let worker_runs_now = runtime()
                    .telemetry
                    .tasks_run_by_workers
                    .load(Ordering::Relaxed);

                // Consider both pending decrease and worker-run increase as forward progress.
                if pending_now < last_pending || worker_runs_now > last_worker_runs {
                    last_pending = pending_now;
                    last_worker_runs = worker_runs_now;
                    stall_spins = 0;
                    failed_assist_cooldown = 0;
                    watchdog_spins = 0;
                    core::hint::spin_loop();
                    continue;
                }

                stall_spins = stall_spins.saturating_add(1);
                watchdog_spins = watchdog_spins.saturating_add(1);
                if stall_spins < MAIN_ASSIST_STALL_SPINS {
                    core::hint::spin_loop();
                    continue;
                }

                stall_spins = 0;

                if failed_assist_cooldown > 0 {
                    failed_assist_cooldown -= 1;
                    core::hint::spin_loop();
                    continue;
                }

                let should_force_assist = watchdog_spins >= MAIN_ASSIST_WATCHDOG_SPINS;
                // Avoid bootstrap steals before workers make first observable progress,
                // unless watchdog fallback is triggered.
                if !should_force_assist && worker_runs_now == worker_runs_at_scope_start {
                    core::hint::spin_loop();
                    continue;
                }

                let assist_started_ns = now_ns();
                let maybe_task = {
                    let runtime_ref = runtime();
                    runtime_ref
                        .telemetry
                        .main_assist_attempt_count
                        .fetch_add(1, Ordering::Relaxed);
                    let mut state = lock_pool_state(&runtime_ref.shared);
                    // Keep main mostly out of worker work unless backlog builds up;
                    // fallback watchdog assist prevents indefinite no-progress loops.
                    let should_try_pop =
                        should_force_assist || state.queue.len() >= MAIN_ASSIST_BACKLOG_THRESHOLD;
                    let task = if should_try_pop {
                        state.queue.pop_front()
                    } else {
                        None
                    };
                    if task.is_some() {
                        runtime_ref
                            .telemetry
                            .queue_pop_count
                            .fetch_add(1, Ordering::Relaxed);
                    } else {
                        runtime_ref
                            .telemetry
                            .queue_empty_poll_count
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    task
                };

                if let Some(task) = maybe_task {
                    let result = catch_unwind(AssertUnwindSafe(|| unsafe { task.run() }));
                    runtime()
                        .telemetry
                        .main_assist_success_count
                        .fetch_add(1, Ordering::Relaxed);
                    runtime()
                        .telemetry
                        .tasks_run_by_main
                        .fetch_add(1, Ordering::Relaxed);
                    let assist_elapsed_ns = now_ns().saturating_sub(assist_started_ns);
                    runtime()
                        .telemetry
                        .main_assist_time_ns
                        .fetch_add(assist_elapsed_ns, Ordering::Relaxed);
                    if let Err(err) = result {
                        scope_runtime.record_panic(err);
                    }
                    scope_runtime.complete_task();
                    failed_assist_cooldown = 0;
                    watchdog_spins = 0;
                    last_pending = scope_runtime.pending.load(Ordering::Acquire);
                    last_worker_runs = runtime()
                        .telemetry
                        .tasks_run_by_workers
                        .load(Ordering::Relaxed);
                } else {
                    let assist_elapsed_ns = now_ns().saturating_sub(assist_started_ns);
                    runtime()
                        .telemetry
                        .main_assist_time_ns
                        .fetch_add(assist_elapsed_ns, Ordering::Relaxed);
                    failed_assist_cooldown = MAIN_ASSIST_FAILED_COOLDOWN_SPINS;
                    core::hint::spin_loop();
                    last_pending = pending_now;
                    last_worker_runs = worker_runs_now;
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut guard = scope_runtime
                .completion_lock
                .lock()
                .expect("poisoned scope completion lock");
            while scope_runtime.pending.load(Ordering::Acquire) != 0 {
                guard = scope_runtime
                    .completion_cv
                    .wait(guard)
                    .expect("poisoned scope completion lock");
            }
        }

        {
            let runtime_ref = runtime();
            let mut state = lock_pool_state(&runtime_ref.shared);
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

impl ParThreadPool for WasmWebPool2 {
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
            let assist_started_ns = now_ns();
            let result = catch_unwind(AssertUnwindSafe(work));
            runtime()
                .telemetry
                .tasks_run_by_main
                .fetch_add(1, Ordering::Relaxed);
            let assist_elapsed_ns = now_ns().saturating_sub(assist_started_ns);
            runtime()
                .telemetry
                .main_assist_time_ns
                .fetch_add(assist_elapsed_ns, Ordering::Relaxed);
            if let Err(err) = result {
                s.runtime().record_panic(err);
            }
            s.runtime().complete_task();
            return;
        }

        let task = Task::new(work);

        {
            let mut state = lock_pool_state(s.shared());
            state.queue.push_back(task);
            let queue_len = state.queue.len();
            runtime()
                .telemetry
                .tasks_enqueued
                .fetch_add(1, Ordering::Relaxed);
            update_high_water(&runtime().telemetry.queue_depth_high_water, queue_len);
        }

        runtime()
            .telemetry
            .notify_count
            .fetch_add(1, Ordering::Relaxed);
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

impl ParThreadPool for &WasmWebPool2 {
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
        <WasmWebPool2 as ParThreadPool>::run_in_scope(s, work)
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
