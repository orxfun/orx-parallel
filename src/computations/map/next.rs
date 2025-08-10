use super::m::M;
use crate::runner::parallel_runner_compute::{next, next_any};
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, O, M1> M<I, O, M1>
where
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
    O: Send,
{
    pub fn next<R>(self) -> (usize, Option<O>)
    where
        R: ParallelRunner,
    {
        let (len, p) = self.len_and_params();
        next::m(R::early_return(p, len), self)
    }

    pub fn next_any<R>(self) -> (usize, Option<O>)
    where
        R: ParallelRunner,
    {
        let (len, p) = self.len_and_params();
        next_any::m(R::early_return(p, len), self)
    }
}
