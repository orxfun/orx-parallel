use crate::computations::{M, X};
use crate::runner::ParallelRunner;
use crate::using::UParCollectIntoCore;
use crate::values::Values;
use crate::values::runner_results::Fallibility;
use orx_concurrent_iter::ConcurrentIter;
use orx_iterable::Collection;
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub trait ParCollectIntoCore<O>: Collection<Item = O> {
    type BridgePinnedVec: IntoConcurrentPinnedVec<O>;

    fn empty(iter_len: Option<usize>) -> Self;

    fn m_collect_into<R, I, M1>(self, m: M<I, O, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        M1: Fn(I::Item) -> O + Sync;

    fn x_collect_into<R, I, Vo, M1>(self, x: X<I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vo + Sync;

    fn x_try_collect_into<R, I, Vo, M1>(
        self,
        x: X<I, Vo, M1>,
    ) -> Result<Self, <Vo::Fallibility as Fallibility>::Error>
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vo + Sync,
        Self: Sized;

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
pub trait ParCollectInto<O>: ParCollectIntoCore<O> + UParCollectIntoCore<O> {}

impl<O, C> ParCollectInto<O> for C where C: ParCollectIntoCore<O> + UParCollectIntoCore<O> {}
