use super::x::X;
use crate::runner::parallel_runner_compute::{next, next_any};
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use crate::values::Values;
use crate::values::runner_results::Infallible;
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
        let (num_threads, result) = next::x(R::early_return(p, len), self);
        let next = match result {
            Ok(x) => x.map(|x| x.1),
        };
        (num_threads, next)
    }

    pub fn next_any<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Vo: Values<Fallibility = Infallible>,
    {
        let (len, p) = self.len_and_params();
        let (num_threads, result) = next_any::x(R::early_return(p, len), self);
        let next = match result {
            Ok(x) => x,
        };
        (num_threads, next)
    }
}
