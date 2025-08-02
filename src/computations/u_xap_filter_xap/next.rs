use crate::computations::{UXfx, Using, Values};
use crate::runner::parallel_runner_compute::{u_next, u_next_any};
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
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
    pub fn next<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        u_next::u_xfx(R::early_return(p, len), self)
    }

    pub fn next_any<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        u_next_any::u_xfx(R::early_return(p, len), self)
    }
}
