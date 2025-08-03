use crate::computations::{M, UM, UX, UXfx, Values, X, Xfx};
use crate::runner::ParallelRunner;
use crate::using::Using;
use orx_concurrent_iter::ConcurrentIter;
use orx_iterable::Collection;
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub trait ParCollectIntoCore<O: Send + Sync>: Collection<Item = O> {
    type BridgePinnedVec: IntoConcurrentPinnedVec<O>;

    fn empty(iter_len: Option<usize>) -> Self;

    fn m_collect_into<R, I, M1>(self, m: M<I, O, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        M1: Fn(I::Item) -> O + Send + Sync;

    fn x_collect_into<R, I, Vo, M1>(self, x: X<I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: Values<Item = O> + Send + Sync,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vo + Send + Sync;

    fn xfx_collect_into<R, I, Vt, Vo, M1, F, M2>(self, xfx: Xfx<I, Vt, Vo, M1, F, M2>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vt: Values + Send + Sync,
        Vt::Item: Send + Sync,
        Vo: Values<Item = O> + Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync;

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

    // test

    #[cfg(test)]
    fn length(&self) -> usize;

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.length() == 0
    }

    #[cfg(test)]
    fn is_equal_to<'a>(&self, b: impl orx_iterable::Iterable<Item = &'a O>) -> bool
    where
        O: PartialEq + 'a,
    {
        let mut b = b.iter();
        for x in self.iter() {
            match b.next() {
                Some(y) if x != y => return false,
                None => return false,
                _ => {}
            }
        }

        b.next().is_none()
    }

    #[cfg(test)]
    fn is_equal_to_ref(&self, b: impl orx_iterable::Iterable<Item = O>) -> bool
    where
        O: PartialEq,
    {
        let mut b = b.iter();
        for x in self.iter() {
            match b.next() {
                Some(y) if x != &y => return false,
                None => return false,
                _ => {}
            }
        }

        b.next().is_none()
    }
}

/// Collection types into which outputs of a parallel computations can be collected into.
pub trait ParCollectInto<O: Send + Sync>: ParCollectIntoCore<O> {}

impl<O: Send + Sync, C: ParCollectIntoCore<O>> ParCollectInto<O> for C {}
