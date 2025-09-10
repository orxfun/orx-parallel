use super::m::M;
use crate::orch::Orchestrator;
use crate::runner::parallel_runner_compute::{next, next_any};
use orx_concurrent_iter::ConcurrentIter;

impl<R, I, O, M1> M<R, I, O, M1>
where
    R: Orchestrator,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
    O: Send,
{
    pub fn next(self) -> (usize, Option<O>) {
        next::m(self)
    }

    pub fn next_any(self) -> (usize, Option<O>) {
        next_any::m(self)
    }
}
