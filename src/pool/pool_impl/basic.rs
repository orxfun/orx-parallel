use crate::NumThreads;
use crate::pool::ParThreadPool;
use crate::pool::env::max_num_threads_by_env_and_resource;
use core::num::NonZeroUsize;
use std::any::Any;
use std::boxed::Box;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::vec::Vec;

struct Inner {
    shared: Arc<WorkerShared>,
    workers: Mutex<Vec<std::thread::JoinHandle<()>>>,
}

struct WorkerShared {
    state: Mutex<WorkerState>,
    cv: Condvar,
}

struct WorkerState {
    shutdown: bool,
    queue: VecDeque<Task>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        {
            let mut state = self.shared.state.lock().expect("poisoned pool lock");
            state.shutdown = true;
            while let Some(task) = state.queue.pop_front() {
                unsafe { task.drop() };
            }
        }
        self.shared.cv.notify_all();

        let mut workers = self.workers.lock().expect("poisoned workers lock");
        for worker in workers.drain(..) {
            let _ = worker.join();
        }
    }
}

struct ScopeRuntime {
    pending: AtomicUsize,
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
        // Decrement and notify under the lock so the main thread cannot exit
        // wait_for_completion (and free this ScopeRuntime) while we still hold a
        // reference to completion_lock / completion_cv.
        let guard = self
            .completion_lock
            .lock()
            .expect("poisoned scope completion lock");
        let prev = self.pending.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            self.completion_cv.notify_all();
        }
        drop(guard);
    }

    fn wait_for_completion(&self) {
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

struct Task {
    data: *mut (),
    run_fn: unsafe fn(*mut ()),
    drop_fn: unsafe fn(*mut ()),
    runtime: *const ScopeRuntime,
}

unsafe impl Send for Task {}

impl Task {
    fn new<W>(work: W, runtime: *const ScopeRuntime) -> Self
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
            runtime,
        }
    }

    unsafe fn run(self) {
        unsafe { (self.run_fn)(self.data) };
    }

    unsafe fn drop(self) {
        unsafe { (self.drop_fn)(self.data) };
    }
}

fn worker_loop(shared: Arc<WorkerShared>) {
    loop {
        let task = {
            let mut state = shared.state.lock().expect("poisoned pool lock");
            loop {
                if state.shutdown {
                    return;
                }

                if let Some(task) = state.queue.pop_front() {
                    break task;
                }

                state = shared.cv.wait(state).expect("poisoned pool lock");
            }
        };

        let runtime = unsafe { &*task.runtime };
        let result = catch_unwind(AssertUnwindSafe(|| unsafe { task.run() }));
        if let Err(err) = result {
            runtime.record_panic(err);
        }
        runtime.complete_task();
    }
}

/// Native standard thread pool with persistent workers.
///
/// This is the default thread pool used when "std" feature is enabled.
/// Note that the thread pool to be used for a parallel computation can be set by the
/// [`runner`] transformation separately for each parallel iterator.
///
/// Value of [`max_num_threads`] is determined as the minimum of:
///
/// * the available parallelism of the host obtained via `std::thread::available_parallelism()`, and
/// * the upper bound set by the environment variable "ORX_NUM_THREADS", when set.
///
/// [`max_num_threads`]: ParThreadPool::max_num_threads
/// [`runner`]: crate::Par::runner
#[derive(Clone)]
pub struct BasicPool {
    max_num_threads: NonZeroUsize,
    inner: Arc<Inner>,
}

impl Default for BasicPool {
    fn default() -> Self {
        Self::new(NumThreads::Auto)
    }
}

impl BasicPool {
    /// Creates a `BasicPool` with persistent worker threads.
    ///
    /// The effective thread count is the minimum of the requested `num_threads`,
    /// the `ORX_NUM_THREADS` environment limit when set, and the
    /// available system parallelism.
    pub fn new(num_threads: impl Into<NumThreads>) -> Self {
        let num_threads = match num_threads.into() {
            NumThreads::Auto => max_num_threads_by_env_and_resource(),
            NumThreads::Max(n) => max_num_threads_by_env_and_resource().min(n),
        };

        let shared = Arc::new(WorkerShared {
            state: Mutex::new(WorkerState {
                shutdown: false,
                queue: VecDeque::new(),
            }),
            cv: Condvar::new(),
        });

        let nt: usize = num_threads.into();
        let mut workers = Vec::with_capacity(nt);
        for _ in 0..nt {
            let shared_cloned = Arc::clone(&shared);
            workers.push(thread::spawn(move || worker_loop(shared_cloned)));
        }

        Self {
            max_num_threads: num_threads,
            inner: Arc::new(Inner {
                shared,
                workers: Mutex::new(workers),
            }),
        }
    }

    fn scoped_computation_impl<'env, 'scope, F>(&'env self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s ScopeRef<'env>) + Send,
    {
        let runtime = ScopeRuntime::new();

        let scope_ref = ScopeRef {
            shared: Arc::as_ptr(&self.inner.shared),
            runtime: &runtime,
            _marker: PhantomData,
        };

        let user_result = catch_unwind(AssertUnwindSafe(|| f(&scope_ref)));

        runtime.wait_for_completion();

        if let Err(err) = user_result {
            resume_unwind(err);
        }

        if let Some(err) = runtime.take_panic() {
            resume_unwind(err);
        }
    }
}

impl ParThreadPool for BasicPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s ScopeRef<'env>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s ScopeRef<'env>) + Send,
    {
        self.scoped_computation_impl(f)
    }

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.runtime().begin_task();

        let task = Task::new(work, s.runtime());
        {
            let mut state = s.shared().state.lock().expect("poisoned pool lock");
            state.queue.push_back(task);
        }
        s.shared().cv.notify_one();
    }
}

impl ParThreadPool for &BasicPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s ScopeRef<'env>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s ScopeRef<'env>) + Send,
    {
        (*self).scoped_computation_impl(f)
    }

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        <BasicPool as ParThreadPool>::run_in_scope(s, work)
    }
}

impl ParThreadPool for &mut BasicPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s ScopeRef<'env>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s ScopeRef<'env>) + Send,
    {
        (*self).scoped_computation_impl(f)
    }

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        <BasicPool as ParThreadPool>::run_in_scope(s, work)
    }
}
