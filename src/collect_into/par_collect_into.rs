use crate::{computations::ParallelRunnerToArchive, parameters::Params};
use orx_concurrent_iter::ConcurrentIter;
use orx_iterable::Collection;
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub trait ParCollectIntoCore<T: Send + Sync>: Collection<Item = T> {
    type BridgePinnedVec: IntoConcurrentPinnedVec<T>;

    fn empty(iter_len: Option<usize>) -> Self;

    fn map_into<I, M, R>(self, params: Params, iter: I, map: M) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> T + Send + Sync + Clone,
        R: ParallelRunnerToArchive;

    // test

    #[cfg(test)]
    fn length(&self) -> usize;

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.length() == 0
    }

    #[cfg(test)]
    fn is_equal_to<'a>(&self, b: impl orx_iterable::Iterable<Item = &'a T>) -> bool
    where
        T: PartialEq + 'a,
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
    fn is_equal_to_ref(&self, b: impl orx_iterable::Iterable<Item = T>) -> bool
    where
        T: PartialEq,
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
