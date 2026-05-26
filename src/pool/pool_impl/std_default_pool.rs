use crate::pool::{ParThreadPool, env::max_num_threads_by_env_and_resource};
use core::num::NonZeroUsize;

/// Native standard thread pool.
///
/// This is the default thread pool used when "std" feature is enabled.
/// Note that the thread pool to be used for a parallel computation can be set by the
/// [`with_runner`] transformation separately for each parallel iterator.
///
/// Uses `std::thread::scope` and `scope.spawn(..)` to distribute work to threads.
///
/// Value of [`max_num_threads`] is determined as the minimum of:
///
/// * the available parallelism of the host obtained via `std::thread::available_parallelism()`, and
/// * the upper bound set by the environment variable "ORX_PARALLEL_MAX_NUM_THREADS", when set.
///
/// [`max_num_threads`]: ParThreadPool::max_num_threads
/// [`with_runner`]: crate::ParIter::with_runner
#[derive(Clone)]
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
    pub fn new(num_threads: NonZeroUsize) -> Self {
        let num_threads = max_num_threads_by_env_and_resource().min(num_threads);
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
