use super::x::X;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, Infallible};
use crate::runner::parallel_runner_compute::reduce;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, Vo, M1> X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    M1: Fn(I::Item) -> Vo + Sync,
{
    pub fn reduce<R, Red>(self, reduce: Red) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Sync,
        Vo: Values<Fallibility = Infallible>,
    {
        let (len, p) = self.len_and_params();
        let (num_threads, Ok(acc)) = reduce::x(R::reduce(p, len), self, reduce);
        (num_threads, acc)
    }

    pub fn try_reduce<R, Red>(
        self,
        reduce: Red,
    ) -> (
        usize,
        Result<Option<Vo::Item>, <Vo::Fallibility as Fallibility>::Error>,
    )
    where
        R: ParallelRunner,
        Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Sync,
    {
        let (len, p) = self.len_and_params();
        reduce::x(R::reduce(p, len), self, reduce)
    }
}
