use crate::{
    computations::{Mfm, Values, M},
    runner::ParallelRunner,
};
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

    fn collect_into<R, I, T, Vt, Vo, M1, F, M2>(
        self,
        mfm: Mfm<I, T, Vt, O, Vo, M1, F, M2>,
        in_input_order: bool,
    ) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vt: Values<Item = T>,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> Vo + Send + Sync;

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

pub trait ParCollectInto<O: Send + Sync>: ParCollectIntoCore<O> {}

impl<O: Send + Sync, C: ParCollectIntoCore<O>> ParCollectInto<O> for C {}
