use super::m::M;
use crate::runner::parallel_runner_compute::reduce;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, O, M1> M<I, O, M1>
where
    I: ConcurrentIter,
    O: Send,
    M1: Fn(I::Item) -> O + Sync,
{
    pub fn reduce<R, X>(self, reduce: X) -> (usize, Option<O>)
    where
        R: ParallelRunner,
        X: Fn(O, O) -> O + Sync,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        reduce::m(R::reduce(p, len), self, reduce)
    }
}
