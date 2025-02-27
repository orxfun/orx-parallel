use crate::{computations::ParallelRunner, parameters::Params};
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub trait ParCollectIntoCore<T: Send + Sync> {
    type BridgePinnedVec: IntoConcurrentPinnedVec<T>;

    fn empty(iter_len: Option<usize>) -> Self;

    fn map_into<I, M, R>(self, params: Params, iter: I, map: M) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> T + Send + Sync + Clone,
        R: ParallelRunner;
}

pub trait ParCollectInto<O: Send + Sync>: ParCollectIntoCore<O> {}

impl<O: Send + Sync, C: ParCollectIntoCore<O>> ParCollectInto<O> for C {}
