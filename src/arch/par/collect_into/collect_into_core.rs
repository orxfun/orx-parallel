use crate::{
    par::{
        par_filtermap_fil::ParFilterMapFilter, par_flatmap_fil::ParFlatMapFilter, par_map::ParMap,
        par_map_fil::ParMapFilter,
    },
    Fallible,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub trait ParCollectIntoCore<O: Send + Sync> {
    type BridgePinnedVec: IntoConcurrentPinnedVec<O>;

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

    fn flatmap_filter_into<I, OI, M, F>(self, par: ParFlatMapFilter<I, O, OI, M, F>) -> Self
    where
        I: ConcurrentIter,
        OI: IntoIterator<Item = O>,
        M: Fn(I::Item) -> OI + Send + Sync,
        F: Fn(&O) -> bool + Send + Sync;

    fn filtermap_filter_into<I, FO, M, F>(self, par: ParFilterMapFilter<I, FO, O, M, F>) -> Self
    where
        I: ConcurrentIter,
        FO: Fallible<O> + Send + Sync,
        M: Fn(I::Item) -> FO + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone;

    fn into_concurrent_bag(self) -> ConcurrentBag<O, Self::BridgePinnedVec>;

    fn from_concurrent_bag(bag: ConcurrentBag<O, Self::BridgePinnedVec>) -> Self;

    fn seq_extend<I: Iterator<Item = O>>(self, iter: I) -> Self
    where
        Self: Sized;
}
