use crate::{
    DefaultRunner, ParallelRunner,
    orch::{Orchestrator, ParThreadPool},
};
use orx_self_or::SoR;
use rayon::ThreadPool;
use std::marker::PhantomData;

pub struct RayonOrchestrator<P, R = DefaultRunner>
where
    R: ParallelRunner,
    P: SoR<ThreadPool> + ParThreadPool,
{
    pool: P,
    runner: PhantomData<R>,
}

impl<R> RayonOrchestrator<ThreadPool, R>
where
    R: ParallelRunner,
{
    pub fn new(pool: ThreadPool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<'a, R> RayonOrchestrator<&'a ThreadPool, R>
where
    R: ParallelRunner,
{
    pub fn new(pool: &'a ThreadPool) -> Self {
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
