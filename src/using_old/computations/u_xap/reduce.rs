use crate::generic_values::Values;
use crate::generic_values::runner_results::Infallible;
use crate::orch::NumSpawned;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use crate::using_old::Using;
use crate::using_old::computations::UX;
use crate::using_old::runner::parallel_runner_compute::u_reduce;
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, Vo, M1> UX<U, I, Vo, M1>
where
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
    pub fn reduce<R, Red>(self, reduce: Red) -> (NumSpawned, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Red: Fn(&mut U::Item, Vo::Item, Vo::Item) -> Vo::Item + Sync,
        Vo: Values<Fallibility = Infallible>,
    {
        let (len, p) = self.len_and_params();
        let (num_threads, Ok(acc)) = u_reduce::u_x(R::reduce(p, len), self, reduce);
        (num_threads, acc)
    }
}
