use super::x::X;
use crate::computations::Values;
use crate::runner::parallel_runner_compute::reduce;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, Vo, M1> X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
{
    pub fn reduce<R, Red>(self, reduce: Red) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        reduce::x(R::reduce(p, len), self, reduce)
    }
}
