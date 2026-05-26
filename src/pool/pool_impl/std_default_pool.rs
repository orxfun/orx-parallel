use crate::NumThreads;
use crate::pool::{ParThreadPool, env::max_num_threads_by_env_and_resource};
use core::num::NonZeroUsize;

/// A _one-time-use_ thread pool.
///
/// This is not an actual thread pool, rather a configuration on number of threads to be spawned.
/// Desired threads will be spawned just before the computation starts and will be released right after.
/// Therefore, it may be considered as a _one-time-use_ thread pool.
///
/// This is also the default thread pool used when "std" feature is enabled.
/// Therefore, "orx-parallel" will not create and hold on to a thread pool unless it is explicitly created.
///
/// On the other hand, in order to reduce the overhead of spawning threads, thread pools can be created
/// using [`Pool`] methods and passed to the parallel iterators using the [`pool`] transformation.
///
/// [`pool`]: crate::Par::pool
/// [`Pool`]: crate::Pool
#[derive(Clone, Copy, Debug)]
pub struct StdDefaultPool {
    num_threads: NonZeroUsize,
}

impl StdDefaultPool {
    /// Assumes (*) a thread pool of `num_threads` threads.
    ///
    /// Note that, this desired number of threads can be overwritten by the following:
    /// - if the system has `n < num_threads` available threads, computation will use `n` threads.
    /// - if ORX_PARALLEL_MAX_NUM_THREADS environment variable exists with value `m < num_threads`,
    ///   computation will use `m` threads.
    ///
    /// (*) This is not an actual thread pool, rather a configuration on number of threads to be spawned.
    /// Desired threads will be spawned just before the computation starts and will be released right after.
    /// Therefore, it may be considered as a _one-time-use_ thread pool.
    pub fn new(num_threads: impl Into<NumThreads>) -> Self {
        let num_threads = match num_threads.into() {
            NumThreads::Auto => max_num_threads_by_env_and_resource(),
            NumThreads::Max(n) => max_num_threads_by_env_and_resource().min(n),
        };
        Self { num_threads }
    }

    /// By default (`StdDefaultPool::default()`), std thread pool assumes that all threads are available
    /// for the parallel computations.
    ///
    /// Constructing the pool with this method makes sure that parallel computations cannot use more than
    /// `max_num_threads` threads.
    pub fn with_max_num_threads(max_num_threads: NonZeroUsize) -> Self {
        let mut pool = Self::default();
        if max_num_threads < pool.num_threads {
            pool.num_threads = max_num_threads;
        }
        pool
    }
}

impl Default for StdDefaultPool {
    fn default() -> Self {
        let num_threads = max_num_threads_by_env_and_resource();
        Self { num_threads }
    }
}

impl ParThreadPool for StdDefaultPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s std::thread::Scope<'s, 'env>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn max_num_threads(&self) -> NonZeroUsize {
        self.num_threads
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s std::thread::Scope<'s, 'env>) + Send,
    {
        std::thread::scope(f)
    }

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.spawn(work);
    }
}

impl ParThreadPool for &StdDefaultPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s std::thread::Scope<'s, 'env>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn max_num_threads(&self) -> NonZeroUsize {
        self.num_threads
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s std::thread::Scope<'s, 'env>) + Send,
    {
        std::thread::scope(f)
    }

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.spawn(work);
    }
}
