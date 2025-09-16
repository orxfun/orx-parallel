use crate::generic_values::Values;
use crate::generic_values::runner_results::Infallible;
use crate::orch::NumSpawned;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use crate::using_old::Using;
use crate::using_old::computations::UX;
use crate::using_old::runner::parallel_runner_compute::{u_next, u_next_any};
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, Vo, M1> UX<U, I, Vo, M1>
where
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    Vo::Item: Send,
{
    pub fn next<R>(self) -> (NumSpawned, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Vo: Values<Fallibility = Infallible>,
    {
        let (len, p) = self.len_and_params();
        let (num_threads, Ok(result)) = u_next::u_x(R::early_return(p, len), self);
        (num_threads, result.map(|x| x.1))
    }

    pub fn next_any<R>(self) -> (NumSpawned, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Vo: Values<Fallibility = Infallible>,
    {
        let (len, p) = self.len_and_params();
        let (num_threads, Ok(next)) = u_next_any::u_x(R::early_return(p, len), self);
        (num_threads, next)
    }
}
