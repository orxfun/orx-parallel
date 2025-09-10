use super::m::M;
use crate::orch::Orchestrator;
use crate::runner::parallel_runner_compute::reduce;
use orx_concurrent_iter::ConcurrentIter;

impl<R, I, O, M1> M<R, I, O, M1>
where
    R: Orchestrator,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(I::Item) -> O + Sync,
{
    pub fn reduce<X>(self, reduce: X) -> (usize, Option<O>)
    where
        X: Fn(O, O) -> O + Sync,
    {
        reduce::m(self, reduce)
    }
}
