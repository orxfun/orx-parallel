use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use crate::using::Using;
use crate::using::computations::UM;
use crate::using::runner::parallel_runner_compute::u_reduce;
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, O, M1> UM<U, I, O, M1>
where
    U: Using,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    pub fn reduce<R, X>(self, reduce: X) -> (usize, Option<O>)
    where
        R: ParallelRunner,
        X: Fn(&mut U::Item, O, O) -> O + Send + Sync,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        u_reduce::u_m(R::reduce(p, len), self, reduce)
    }
}
