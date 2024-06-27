use super::par_filtermap::ParFilterMap;
use super::{par_flatmap_fil::ParFlatMapFilter, reduce::Reduce};
use crate::{core::default_fns::no_filter, Params};
use crate::{Fallible, ParCollectInto, ParIter};
use orx_concurrent_iter::{ConIterOfVec, ConcurrentIter, IntoConcurrentIter};
use orx_split_vec::SplitVec;
use std::fmt::Debug;
use std::iter::Map;

/// An iterator that maps the elements of the iterator with a given map function.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`ParMap::num_threads`] and [`ParMap::chunk_size`] methods.
pub struct ParFlatMap<I, O, OI, M>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync + Clone,
{
    iter: I,
    params: Params,
    fmap: M,
}

impl<I, O, OI, M> ParIter for ParFlatMap<I, O, OI, M>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
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
        O2: Send + Sync + Debug,
        M2: Fn(Self::Item) -> O2 + Send + Sync + Clone,
    {
        let (params, iter, map1) = (self.params, self.iter, self.fmap);
        let composed = move |x| {
            let map1 = map1.clone();
            let values = map1(x);
            values.into_iter().map(map.clone())
        };
        ParFlatMap::new(iter, params, composed)
    }

    fn flat_map<O2, OI2, FM>(self, flat_map: FM) -> ParFlatMap<ConIterOfVec<O>, O2, OI2, FM>
    where
        O2: Send + Sync + Debug,
        OI2: IntoIterator<Item = O2>,
        FM: Fn(Self::Item) -> OI2 + Send + Sync + Clone,
    {
        // todo! could fmap's be composed?
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParFlatMap::new(iter, params, flat_map)
    }

    fn filter<F>(self, filter: F) -> ParFlatMapFilter<I, O, OI, M, F>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync,
    {
        ParFlatMapFilter::new(self.iter, self.params, self.fmap, filter)
    }

    fn filter_map<O2, FO, FM>(self, filter_map: FM) -> ParFilterMap<ConIterOfVec<O>, FO, O2, FM>
    where
        O2: Send + Sync + Debug,
        FO: Fallible<O2> + Send + Sync + Debug,
        FM: Fn(Self::Item) -> FO + Send + Sync + Clone,
    {
        self.filter(no_filter).filter_map(filter_map)
    }

    // reduce

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

    fn collect_x_vec(self) -> Vec<Self::Item> {
        self.filter(no_filter).collect_x_vec()
    }

    fn collect_x(self) -> SplitVec<Self::Item> {
        self.filter(no_filter).collect_x()
    }

    fn collect_x_into<B: ParCollectInto<Self::Item>>(self, output: B) -> B {
        self.filter(no_filter).collect_x_into(output)
    }
}

impl<I, O, OI, M> ParFlatMap<I, O, OI, M>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync + Clone,
{
    pub(crate) fn new(iter: I, params: Params, fmap: M) -> Self {
        Self { iter, params, fmap }
    }
}

impl<I, O, OI, M> Reduce<O> for ParFlatMap<I, O, OI, M>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync + Clone,
    O:,
{
    fn reduce<R>(self, reduce: R) -> Option<O>
    where
        R: Fn(O, O) -> O + Send + Sync,
    {
        self.filter(no_filter).reduce(reduce)
    }
}
