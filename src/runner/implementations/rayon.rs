use crate::{
    DefaultExecutor, ParallelExecutor, par_thread_pool::ParThreadPool, runner::ParallelRunner,
};
use core::{marker::PhantomData, num::NonZeroUsize};
use orx_self_or::SoR;
use rayon::ThreadPool;

// POOL

impl ParThreadPool for ThreadPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s rayon::Scope<'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.spawn(move |_| work());
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s rayon::Scope<'scope>) + Send,
    {
        self.scope(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.current_num_threads().max(1)).expect(">0")
    }
}

impl ParThreadPool for &rayon::ThreadPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s rayon::Scope<'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.spawn(move |_| work());
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s rayon::Scope<'scope>) + Send,
    {
        self.scope(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.current_num_threads().max(1)).expect(">0")
    }
}

// RUNNER

/// Parallel runner using threads provided by rayon::ThreadPool.
pub struct RunnerWithRayonPool<P, R = DefaultExecutor>
where
    R: ParallelExecutor,
    P: SoR<ThreadPool> + ParThreadPool,
{
    pool: P,
    runner: PhantomData<R>,
}

impl From<ThreadPool> for RunnerWithRayonPool<ThreadPool, DefaultExecutor> {
    fn from(pool: ThreadPool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<'a> From<&'a ThreadPool> for RunnerWithRayonPool<&'a ThreadPool, DefaultExecutor> {
    fn from(pool: &'a ThreadPool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<P, R> ParallelRunner for RunnerWithRayonPool<P, R>
where
    R: ParallelExecutor,
    P: SoR<ThreadPool> + ParThreadPool,
{
    type Executor = R;

    type ThreadPool = P;

    fn thread_pool(&self) -> &Self::ThreadPool {
        &self.pool
    }

    fn thread_pool_mut(&mut self) -> &mut Self::ThreadPool {
        &mut self.pool
    }
}
