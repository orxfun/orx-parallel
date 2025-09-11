use super::x::X;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, Infallible};
use crate::orch::Orchestrator;
use crate::runner::parallel_runner_compute::reduce;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<R, I, Vo, X1> X<R, I, Vo, X1>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
{
    pub fn try_reduce<Red>(self, reduce: Red) -> (usize, ResultTryReduce<Vo>)
    where
        Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Sync,
    {
        todo!()
        // let (len, p) = self.len_and_params();
        // reduce::x(R::reduce(p, len), self, reduce)
    }
}

type ResultTryReduce<Vo> =
    Result<Option<<Vo as Values>::Item>, <<Vo as Values>::Fallibility as Fallibility>::Error>;
