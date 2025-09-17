use crate::{
    DefaultExecutor, ParallelExecutor, par_thread_pool::ParThreadPool, runner::ParallelRunner,
};
use core::{marker::PhantomData, num::NonZeroUsize};
use orx_self_or::SoM;
use scoped_threadpool::Pool;

// POOL

impl ParThreadPool for Pool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s scoped_threadpool::Scope<'env, 'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.execute(work);
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s scoped_threadpool::Scope<'env, 'scope>) + Send,
    {
        self.scoped(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        NonZeroUsize::new((self.thread_count() as usize).max(1)).expect(">0")
    }
}

impl<'a> ParThreadPool for &'a mut Pool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s scoped_threadpool::Scope<'env, 'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.execute(work);
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s scoped_threadpool::Scope<'env, 'scope>) + Send,
    {
        self.scoped(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        NonZeroUsize::new((self.thread_count() as usize).max(1)).expect(">0")
    }
}

// ORCH

pub struct ScopedThreadPoolOrchestrator<P, R = DefaultExecutor>
where
    R: ParallelExecutor,
    P: SoM<Pool> + ParThreadPool,
{
    pool: P,
    runner: PhantomData<R>,
}

impl<R> From<Pool> for ScopedThreadPoolOrchestrator<Pool, R>
where
    R: ParallelExecutor,
{
    fn from(pool: Pool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<'a, R> From<&'a mut Pool> for ScopedThreadPoolOrchestrator<&'a mut Pool, R>
where
    R: ParallelExecutor,
{
    fn from(pool: &'a mut Pool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<P, R> ParallelRunner for ScopedThreadPoolOrchestrator<P, R>
where
    R: ParallelExecutor,
    P: SoM<Pool> + ParThreadPool,
{
    type Runner = R;

    type ThreadPool = P;

    fn thread_pool(&self) -> &Self::ThreadPool {
        &self.pool
    }

    fn thread_pool_mut(&mut self) -> &mut Self::ThreadPool {
        &mut self.pool
    }
}
