use crate::{computations::ParallelRunner, parameters::Params};
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub trait ParCollectIntoCore<T: Send + Sync> {
    type BridgePinnedVec: IntoConcurrentPinnedVec<T>;

    fn map_into<I, M, R>(self, params: Params, iter: I, map: M) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> T + Send + Sync + Clone,
        R: ParallelRunner;

    // /// Performs the parallel map operation, collecting the results into this collection.
    // fn map_into<I, M>(self, par_map: ParMap<I, O, M>) -> Self
    // where
    //     I: ConcurrentIter,
    //     M: Fn(I::Item) -> O + Send + Sync + Clone;

    // fn map_filter_into<I, M, F>(self, par: ParMapFilter<I, O, M, F>) -> Self
    // where
    //     I: ConcurrentIter,
    //     M: Fn(I::Item) -> O + Send + Sync + Clone,
    //     F: Fn(&O) -> bool + Send + Sync + Clone;

    // fn flatmap_filter_into<I, OI, M, F>(self, par: ParFlatMapFilter<I, O, OI, M, F>) -> Self
    // where
    //     I: ConcurrentIter,
    //     OI: IntoIterator<Item = O>,
    //     M: Fn(I::Item) -> OI + Send + Sync,
    //     F: Fn(&O) -> bool + Send + Sync;

    // fn filtermap_filter_into<I, FO, M, F>(self, par: ParFilterMapFilter<I, FO, O, M, F>) -> Self
    // where
    //     I: ConcurrentIter,
    //     FO: Fallible<O> + Send + Sync,
    //     M: Fn(I::Item) -> FO + Send + Sync + Clone,
    //     F: Fn(&O) -> bool + Send + Sync + Clone;

    // fn into_concurrent_bag(self) -> ConcurrentBag<O, Self::BridgePinnedVec>;

    // fn from_concurrent_bag(bag: ConcurrentBag<O, Self::BridgePinnedVec>) -> Self;

    // fn seq_extend<I: Iterator<Item = O>>(self, iter: I) -> Self
    // where
    //     Self: Sized;
}

pub trait ParCollectInto<O: Send + Sync>: ParCollectIntoCore<O> {}

impl<O: Send + Sync, C: ParCollectIntoCore<O>> ParCollectInto<O> for C {}
