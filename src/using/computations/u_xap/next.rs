use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use crate::using::Using;
use crate::using::computations::UX;
use crate::using::runner::parallel_runner_compute::{u_next, u_next_any};
use crate::generic_values::Values;
use crate::generic_values::runner_results::Infallible;
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, Vo, M1> UX<U, I, Vo, M1>
where
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    Vo::Item: Send,
{
    pub fn next<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Vo: Values<Fallibility = Infallible>,
    {
        let (len, p) = self.len_and_params();
        let (num_threads, result) = u_next::u_x(R::early_return(p, len), self);
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
        let (num_threads, result) = u_next_any::u_x(R::early_return(p, len), self);
        let next = match result {
            Ok(x) => x,
        };
        (num_threads, next)
    }
}
