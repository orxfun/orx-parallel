use crate::computations::Values;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use crate::using::Using;
use crate::using::computations::UXfx;
use crate::using::runner::parallel_runner_compute::u_reduce;
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, Vt, Vo, M1, F, M2> UXfx<U, I, Vt, Vo, M1, F, M2>
where
    U: Using,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vt + Send + Sync,
    F: Fn(&mut U::Item, &Vt::Item) -> bool + Send + Sync,
    M2: Fn(&mut U::Item, Vt::Item) -> Vo + Send + Sync,
{
    pub fn reduce<R, Red>(self, reduce: Red) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Red: Fn(&mut U::Item, Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        u_reduce::u_xfx(R::reduce(p, len), self, reduce)
    }
}
