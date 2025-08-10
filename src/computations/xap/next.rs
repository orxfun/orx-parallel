use super::x::X;
use crate::runner::ParallelRunnerCompute;
use crate::runner::parallel_runner_compute::{next, next_any};
use crate::{ParallelRunner, computations::Values};
use orx_concurrent_iter::ConcurrentIter;

impl<I, Vo, M1> X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values,
    M1: Fn(I::Item) -> Vo + Sync,
    Vo::Item: Send,
{
    pub fn next<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        let (len, p) = self.len_and_params();
        next::x(R::early_return(p, len), self)
    }

    pub fn next_any<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        let (len, p) = self.len_and_params();
        next_any::x(R::early_return(p, len), self)
    }
}
