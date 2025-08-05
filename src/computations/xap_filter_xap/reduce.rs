use super::xfx::Xfx;
use crate::computations::Values;
use crate::runner::parallel_runner_compute::reduce;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, Vt, Vo, M1, F, M2> Xfx<I, Vt, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    pub fn reduce<R, Red>(self, reduce: Red) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        reduce::xfx(R::reduce(p, len), self, reduce)
    }
}
