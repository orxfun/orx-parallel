use crate::collect_into::ParCollectIntoCore;
use crate::computations::Values;
use crate::runner::ParallelRunner;
use crate::using::Using;
use crate::using::computations::{UM, UX, UXfx};
use orx_concurrent_iter::ConcurrentIter;

pub trait UParCollectIntoCore<O: Send + Sync>: ParCollectIntoCore<O> {
    fn u_m_collect_into<R, U, I, M1>(self, m: UM<U, I, O, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync;

    fn u_x_collect_into<R, U, I, Vo, M1>(self, x: UX<U, I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        Vo: Values<Item = O> + Send + Sync,
        Vo::Item: Send + Sync,
        M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync;

    fn u_xfx_collect_into<R, U, I, Vt, Vo, M1, F, M2>(
        self,
        xfx: UXfx<U, I, Vt, Vo, M1, F, M2>,
    ) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        Vt: Values + Send + Sync,
        Vt::Item: Send + Sync,
        Vo: Values<Item = O> + Send + Sync,
        M1: Fn(&mut U::Item, I::Item) -> Vt + Send + Sync,
        F: Fn(&mut U::Item, &Vt::Item) -> bool + Send + Sync,
        M2: Fn(&mut U::Item, Vt::Item) -> Vo + Send + Sync;
}
