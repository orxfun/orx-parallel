use super::xfx::Xfx;
use crate::computations::Values;
use crate::runner::parallel_runner_compute::{next, next_any};
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, Vt, Vo, M1, F, M2> Xfx<I, Vt, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send,
    M1: Fn(I::Item) -> Vt + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    pub fn next<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        next::xfx(R::early_return(p, len), self)
    }

    pub fn next_any<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        next_any::xfx(R::early_return(p, len), self)
    }
}
