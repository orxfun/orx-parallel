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
    pub fn try_next<R>(self) -> (usize, ResultTryNext<Vo>)
    where
        R: ParallelRunner,
    {
        todo!()
        // let (len, p) = self.len_and_params();
        // let (num_threads, result) = next::x(R::early_return(p, len), self);
        // let result = result.map(|x| x.map(|y| y.1));
        // (num_threads, result)
    }

    pub fn try_next_any<R>(self) -> (usize, ResultTryNext<Vo>)
    where
        R: ParallelRunner,
    {
        todo!()
        // let (len, p) = self.len_and_params();
        // let (num_threads, result) = next_any::x(R::early_return(p, len), self);
        // (num_threads, result)
    }
}

type ResultTryNext<Vo> =
    Result<Option<<Vo as Values>::Item>, <<Vo as Values>::Fallibility as Fallibility>::Error>;
