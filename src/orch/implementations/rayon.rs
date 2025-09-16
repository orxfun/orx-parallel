use crate::{
    DefaultRunner, ParallelRunner,
    orch::{Orchestrator, ParThreadPool},
};
use orx_self_or::SoR;
use rayon::ThreadPool;
use std::{marker::PhantomData, num::NonZeroUsize};

// POOL

impl ParThreadPool for ThreadPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s rayon::Scope<'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: &'env W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Sync + 'scope + 'env,
    {
        s.spawn(|_| work());
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

impl<'a> ParThreadPool for &'a rayon::ThreadPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s rayon::Scope<'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: &'env W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Sync + 'scope + 'env,
    {
        s.spawn(|_| work());
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

// ORCH

pub struct RayonOrchestrator<P, R = DefaultRunner>
where
    R: ParallelRunner,
    P: SoR<ThreadPool> + ParThreadPool,
{
    pool: P,
    runner: PhantomData<R>,
}

impl<R> From<ThreadPool> for RayonOrchestrator<ThreadPool, R>
where
    R: ParallelRunner,
{
    fn from(pool: ThreadPool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<'a, R> From<&'a ThreadPool> for RayonOrchestrator<&'a ThreadPool, R>
where
    R: ParallelRunner,
{
    fn from(pool: &'a ThreadPool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<P, R> Orchestrator for RayonOrchestrator<P, R>
where
    R: ParallelRunner,
    P: SoR<ThreadPool> + ParThreadPool,
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
