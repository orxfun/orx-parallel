use super::{par_fmap_fil::ParFMapFilter, reduce::Reduce};
use crate::{core::default_fns::no_filter, Params};
use crate::{ParCollectInto, ParIter};
use orx_concurrent_iter::{ConIterOfVec, ConcurrentIter, IntoConcurrentIter};
use orx_split_vec::SplitVec;
use std::fmt::Debug;
use std::iter::Map;

/// An iterator that maps the elements of the iterator with a given map function.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`ParMap::num_threads`] and [`ParMap::chunk_size`] methods.
pub struct ParFMap<I, O, OI, M>
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

impl<I, O, OI, M> ParIter for ParFMap<I, O, OI, M>
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

    fn map<O2, M2>(
        self,
        map: M2,
    ) -> ParFMap<
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
        let composed = move |x: I::Item| {
            let map1 = map1.clone();
            let values = map1(x);
            values.into_iter().map(map.clone())
        };
        ParFMap::new(iter, params, composed)
    }

    fn flat_map<O2, OI2, FM>(self, fmap: FM) -> ParFMap<ConIterOfVec<O>, O2, OI2, FM>
    where
        O2: Send + Sync + Debug,
        OI2: IntoIterator<Item = O2>,
        FM: Fn(Self::Item) -> OI2 + Send + Sync + Clone,
    {
        // todo! could fmap's be composed?
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParFMap::new(iter, params, fmap)
    }

    fn filter<F>(self, filter: F) -> ParFMapFilter<I, O, OI, M, F>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync,
    {
        ParFMapFilter::new(self.iter, self.params, self.fmap, filter)
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

impl<I, O, OI, M> ParFMap<I, O, OI, M>
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

impl<I, O, OI, M> Reduce<O> for ParFMap<I, O, OI, M>
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
