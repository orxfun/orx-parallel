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
    Vt: Values,
    Vo: Values,
    M1: Fn(&mut U::Item, I::Item) -> Vt + Sync,
    F: Fn(&mut U::Item, &Vt::Item) -> bool + Sync,
    M2: Fn(&mut U::Item, Vt::Item) -> Vo + Sync,
    Vo::Item: Send,
{
    pub fn reduce<R, Red>(self, reduce: Red) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Red: Fn(&mut U::Item, Vo::Item, Vo::Item) -> Vo::Item + Sync,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        u_reduce::u_xfx(R::reduce(p, len), self, reduce)
    }
}
