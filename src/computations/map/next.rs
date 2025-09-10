use super::m::M;
use crate::orch::Orchestrator;
use crate::runner::parallel_runner_compute::{next, next_any};
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<R, I, O, M1> M<R, I, O, M1>
where
    R: Orchestrator,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
    O: Send,
{
    pub fn next(self) -> (usize, Option<O>) {
        let (len, p) = self.len_and_params();
        next::m(R::Runner::early_return(p, len), self)
    }

    pub fn next_any(self) -> (usize, Option<O>) {
        let (len, p) = self.len_and_params();
        next_any::m(R::Runner::early_return(p, len), self)
    }
}
