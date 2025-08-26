use super::x::X;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, Infallible};
use crate::runner::parallel_runner_compute::{next, next_any};
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
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
        Vo: Values<Fallibility = Infallible>,
    {
        let (len, p) = self.len_and_params();
        let (num_threads, Ok(result)) = next::x(R::early_return(p, len), self);
        (num_threads, result.map(|x| x.1))
    }

    pub fn next_any<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Vo: Values<Fallibility = Infallible>,
    {
        let (len, p) = self.len_and_params();
        let (num_threads, Ok(next)) = next_any::x(R::early_return(p, len), self);
        (num_threads, next)
    }

    pub fn try_next<R>(
        self,
    ) -> (
        usize,
        Result<Option<Vo::Item>, <Vo::Fallibility as Fallibility>::Error>,
    )
    where
        R: ParallelRunner,
    {
        let (len, p) = self.len_and_params();
        let (num_threads, result) = next::x(R::early_return(p, len), self);
        let result = result.map(|x| x.map(|y| y.1));
        (num_threads, result)
    }

    pub fn try_next_any<R>(
        self,
    ) -> (
        usize,
        Result<Option<Vo::Item>, <Vo::Fallibility as Fallibility>::Error>,
    )
    where
        R: ParallelRunner,
    {
        let (len, p) = self.len_and_params();
        let (num_threads, result) = next_any::x(R::early_return(p, len), self);
        (num_threads, result)
    }
}
