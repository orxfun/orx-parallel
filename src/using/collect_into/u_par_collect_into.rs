use crate::collect_into::ParCollectIntoCore;
use crate::computations::Values;
use crate::runner::ParallelRunner;
use crate::using::Using;
use crate::using::computations::{UM, UX, UXfx};
use orx_concurrent_iter::ConcurrentIter;

pub trait UParCollectIntoCore<O>: ParCollectIntoCore<O> {
    fn u_m_collect_into<R, U, I, M1>(self, m: UM<U, I, O, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        M1: Fn(&mut U::Item, I::Item) -> O + Sync;

    fn u_x_collect_into<R, U, I, Vo, M1>(self, x: UX<U, I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        Vo: Values<Item = O>,
        M1: Fn(&mut U::Item, I::Item) -> Vo + Sync;

    fn u_xfx_collect_into<R, U, I, Vt, Vo, M1, F, M2>(
        self,
        xfx: UXfx<U, I, Vt, Vo, M1, F, M2>,
    ) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values<Item = O>,
        M1: Fn(&mut U::Item, I::Item) -> Vt + Sync,
        F: Fn(&mut U::Item, &Vt::Item) -> bool + Sync,
        M2: Fn(&mut U::Item, Vt::Item) -> Vo + Sync;
}
