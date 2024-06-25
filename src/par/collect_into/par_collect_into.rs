use crate::{par::par_fmap_fil::ParFMapFilter, ParMap, ParMapFilter};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::PinnedVec;

pub trait ParCollectInto<O: Send + Sync + Default> {
    type BridgePinnedVec: PinnedVec<O>;

    /// Performs the parallel map operation, collecting the results into this collection.
    fn map_into<I, M>(self, par_map: ParMap<I, O, M>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone;

    fn map_filter_into<I, M, F>(self, par: ParMapFilter<I, O, M, F>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone;

    fn fmap_filter_into<I, OI, M, F>(self, par: ParFMapFilter<I, O, OI, M, F>) -> Self
    where
        I: ConcurrentIter,
        OI: IntoIterator<Item = O>,
        M: Fn(I::Item) -> OI + Send + Sync,
        F: Fn(&O) -> bool + Send + Sync;

    fn into_concurrent_bag(self) -> ConcurrentBag<O, Self::BridgePinnedVec>;

    fn from_concurrent_bag(bag: ConcurrentBag<O, Self::BridgePinnedVec>) -> Self;
}
