use crate::par_thread_pool::ParThreadPool;
use core::num::NonZeroUsize;

const MAX_UNSET_NUM_THREADS: NonZeroUsize = NonZeroUsize::new(8).expect(">0");

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
pub struct StdDefaultPool {
    max_num_threads: NonZeroUsize,
}

impl Default for StdDefaultPool {
    fn default() -> Self {
        let env_max_num_threads = crate::env::max_num_threads_by_env_variable();

        let ava_max_num_threads = std::thread::available_parallelism().ok();

        let max_num_threads = match (env_max_num_threads, ava_max_num_threads) {
            (Some(env), Some(ava)) => env.min(ava),
            (Some(env), None) => env,
            (None, Some(ava)) => ava,
            (None, None) => MAX_UNSET_NUM_THREADS,
        };

        Self { max_num_threads }
    }
}

impl ParThreadPool for StdDefaultPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s std::thread::Scope<'s, 'env>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
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
        self.max_num_threads
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
