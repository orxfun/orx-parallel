use super::par_filtermap::ParFilterMap;
use super::par_flatmap_fil::ParFlatMapFilter;
use crate::{core::default_fns::no_filter, Params};
use crate::{Fallible, ParCollectInto, ParIter};
use orx_concurrent_iter::{ConIterOfVec, ConcurrentIter};
use orx_split_vec::{Recursive, SplitVec};
use std::iter::Map;

/// A parallel iterator.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`crate::ParIter::num_threads`] and [`crate::ParIter::chunk_size`] methods.
pub struct ParFlatMap<I, O, OI, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync + Clone,
{
    iter: I,
    params: Params,
    flat_map: M,
}

impl<I, O, OI, M> ParFlatMap<I, O, OI, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync + Clone,
{
    pub(crate) fn new(iter: I, params: Params, flat_map: M) -> Self {
        Self {
            iter,
            params,
            flat_map,
        }
    }

    fn destruct(self) -> (Params, I, M) {
        (self.params, self.iter, self.flat_map)
    }
}

impl<I, O, OI, M> ParIter for ParFlatMap<I, O, OI, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync + Clone,
{
    type Item = O;

    fn params(&self) -> Params {
        self.params
    }

    fn num_threads(mut self, num_threads: impl Into<crate::NumThreads>) -> Self {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<crate::ChunkSize>) -> Self {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    // transform

    fn map<O2, M2>(
        self,
        map: M2,
    ) -> ParFlatMap<
        I,
        O2,
        Map<<OI as IntoIterator>::IntoIter, M2>,
        impl Fn(<I as ConcurrentIter>::Item) -> Map<<OI as IntoIterator>::IntoIter, M2> + Clone,
    >
    where
        O2: Send + Sync,
        M2: Fn(Self::Item) -> O2 + Send + Sync + Clone,
    {
        let (params, iter, flat_map) = self.destruct();

        let composed_flat_map = move |x| {
            let values = flat_map(x);
            values.into_iter().map(map.clone())
        };
        ParFlatMap::new(iter, params, composed_flat_map)
    }

    fn flat_map<O2, OI2, FM>(self, flat_map: FM) -> impl ParIter<Item = O2>
    where
        O2: Send + Sync,
        OI2: IntoIterator<Item = O2>,
        FM: Fn(Self::Item) -> OI2 + Send + Sync + Clone,
    {
        let (params, iter, flat_map1) = self.destruct();

        let composed_flat_map = move |x| {
            let values = flat_map1(x);
            values.into_iter().flat_map(flat_map.clone())
        };

        ParFlatMap::new(iter, params, composed_flat_map)
    }

    fn filter<F>(self, filter: F) -> ParFlatMapFilter<I, O, OI, M, F>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync,
    {
        ParFlatMapFilter::new(self.iter, self.params, self.flat_map, filter)
    }

    fn filter_map<O2, FO, FM>(self, filter_map: FM) -> ParFilterMap<ConIterOfVec<O>, FO, O2, FM>
    where
        O2: Send + Sync,
        FO: Fallible<O2> + Send + Sync,
        FM: Fn(Self::Item) -> FO + Send + Sync + Clone,
    {
        self.filter(no_filter).filter_map(filter_map)
    }

    // reduce

    fn reduce<R>(self, reduce: R) -> Option<Self::Item>
    where
        R: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync + Clone,
    {
        self.filter(no_filter).reduce(reduce)
    }

    fn count(self) -> usize {
        self.filter(no_filter).count()
    }

    // find
    fn find<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        self.filter(no_filter).find(predicate)
    }

    fn first(self) -> Option<Self::Item> {
        self.filter(no_filter).first()
    }

    // collect

    fn collect_vec(self) -> Vec<Self::Item> {
        self.filter(no_filter).collect_vec()
    }

    fn collect(self) -> SplitVec<Self::Item> {
        self.filter(no_filter).collect()
    }

    fn collect_into<C: ParCollectInto<Self::Item>>(self, output: C) -> C {
        self.filter(no_filter).collect_into(output)
    }

    fn collect_x(self) -> SplitVec<Self::Item, Recursive> {
        self.filter(no_filter).collect_x()
    }
}
