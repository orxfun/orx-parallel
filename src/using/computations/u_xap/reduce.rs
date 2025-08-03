use crate::computations::Values;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use crate::using::Using;
use crate::using::computations::UX;
use crate::using::runner::parallel_runner_compute::u_reduce;
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, Vo, M1> UX<U, I, Vo, M1>
where
    U: Using,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
{
    pub fn reduce<R, Red>(self, reduce: Red) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        u_reduce::u_x(R::reduce(p, len), self, reduce)
    }
}
