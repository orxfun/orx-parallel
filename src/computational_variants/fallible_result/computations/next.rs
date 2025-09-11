use super::x::X;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, Infallible};
use crate::orch::Orchestrator;
use crate::runner::parallel_runner_compute::{next, next_any};
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<R, I, Vo, M1> X<R, I, Vo, M1>
where
    R: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
    M1: Fn(I::Item) -> Vo + Sync,
    Vo::Item: Send,
{
    pub fn try_next(self) -> (usize, ResultTryNext<Vo>) {
        todo!()
        // let (len, p) = self.len_and_params();
        // let (num_threads, result) = next::x(R::early_return(p, len), self);
        // let result = result.map(|x| x.map(|y| y.1));
        // (num_threads, result)
    }

    pub fn try_next_any(self) -> (usize, ResultTryNext<Vo>) {
        todo!()
        // let (len, p) = self.len_and_params();
        // let (num_threads, result) = next_any::x(R::early_return(p, len), self);
        // (num_threads, result)
    }
}

type ResultTryNext<Vo> =
    Result<Option<<Vo as Values>::Item>, <<Vo as Values>::Fallibility as Fallibility>::Error>;
