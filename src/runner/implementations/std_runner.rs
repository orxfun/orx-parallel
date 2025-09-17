use crate::ParallelExecutor;
use crate::par_thread_pool::ParThreadPool;
use crate::{DefaultExecutor, runner::ParallelRunner};
use core::marker::PhantomData;
use core::num::NonZeroUsize;

// POOL

const MAX_UNSET_NUM_THREADS: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(8) };

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
        std::thread::scope(|s| f(&s))
    }

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.spawn(move || work());
    }
}

// RUNNER

/// Parallel runner using std threads.
pub struct StdRunner<E: ParallelExecutor = DefaultExecutor> {
    pool: StdDefaultPool,
    executor: PhantomData<E>,
}

impl<E: ParallelExecutor> Default for StdRunner<E> {
    fn default() -> Self {
        Self {
            pool: Default::default(),
            executor: PhantomData,
        }
    }
}

impl<E: ParallelExecutor> ParallelRunner for StdRunner<E> {
    type Executor = E;

    type ThreadPool = StdDefaultPool;

    fn thread_pool(&self) -> &Self::ThreadPool {
        &self.pool
    }

    fn thread_pool_mut(&mut self) -> &mut Self::ThreadPool {
        &mut self.pool
    }
}
