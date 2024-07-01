use super::{par_filtermap_fil::ParFilterMapFilter, par_flatmap::ParFlatMap, reduce::Reduce};
use crate::{
    core::default_fns::no_filter, fn_sync::FnSync, ChunkSize, Fallible, NumThreads, ParCollectInto,
    ParIter, Params,
};
use orx_concurrent_iter::{ConcurrentIter, IntoConcurrentIter};
use orx_split_vec::SplitVec;
use std::{fmt::Debug, marker::PhantomData};

/// A parallel iterator.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`crate::ParIter::num_threads`] and [`crate::ParIter::chunk_size`] methods.
pub struct ParFilterMap<I, FO, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    FO: Fallible<O> + Send + Sync + Debug,
    M: Fn(I::Item) -> FO + FnSync,
{
    iter: I,
    params: Params,
    filter_map: M,
    phantom: PhantomData<O>,
}

impl<I, FO, O, M> ParFilterMap<I, FO, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    FO: Fallible<O> + Send + Sync + Debug,
    M: Fn(I::Item) -> FO + FnSync,
{
    pub(crate) fn new(iter: I, params: Params, filter_map: M) -> Self {
        Self {
            iter,
            params,
            filter_map,
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, M) {
        (self.params, self.iter, self.filter_map)
    }
}

impl<I, FO, O, M> ParIter for ParFilterMap<I, FO, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    FO: Fallible<O> + Send + Sync + Debug,
    M: Fn(I::Item) -> FO + FnSync,
{
    type Item = O;

    fn params(&self) -> Params {
        self.params
    }

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    // transform

    fn map<O2, M2>(
        self,
        map: M2,
    ) -> ParFilterMap<I, Option<O2>, O2, impl Fn(<I as ConcurrentIter>::Item) -> Option<O2> + FnSync>
    where
        O2: Send + Sync + Debug,
        M2: Fn(Self::Item) -> O2 + FnSync,
    {
        let (params, iter, filter_map) = self.destruct();

        let composed_filter_map = move |x: I::Item| filter_map(x).into_option().map(map.clone());
        ParFilterMap::new(iter, params, composed_filter_map)
    }

    fn flat_map<O2, OI, FM>(self, flat_map: FM) -> impl ParIter<Item = O2>
    where
        O2: Send + Sync + Debug,
        OI: IntoIterator<Item = O2>,
        FM: Fn(Self::Item) -> OI + FnSync,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParFlatMap::new(iter, params, flat_map)
    }

    fn filter<F2>(self, filter: F2) -> impl ParIter<Item = Self::Item>
    where
        F2: Fn(&Self::Item) -> bool + FnSync,
    {
        ParFilterMapFilter::new(self.iter, self.params, self.filter_map, filter)
    }

    fn filter_map<O2, FO2, FM>(
        self,
        filter_map: FM,
    ) -> ParFilterMap<I, Option<O2>, O2, impl Fn(<I as ConcurrentIter>::Item) -> Option<O2> + FnSync>
    where
        O2: Send + Sync + Debug,
        FO2: Fallible<O2> + Send + Sync + Debug,
        FM: Fn(Self::Item) -> FO2 + FnSync,
    {
        let (params, iter, filter_map1) = self.destruct();

        let composed_filter_map = move |x| {
            let mapped = filter_map1(x);
            match mapped.has_value() {
                false => None,
                true => {
                    let value = mapped.value();
                    filter_map(value).into_option()
                }
            }
        };
        ParFilterMap::new(iter, params, composed_filter_map)
    }

    // reduce

    fn count(self) -> usize {
        self.filter(no_filter).count()
    }

    // find

    fn find<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + FnSync,
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
}

impl<I, FO, O, M> Reduce<O> for ParFilterMap<I, FO, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    FO: Fallible<O> + Send + Sync + Debug,
    M: Fn(I::Item) -> FO + FnSync,
{
    fn reduce<R>(self, reduce: R) -> Option<O>
    where
        R: Fn(O, O) -> O + Send + Sync,
    {
        self.filter(no_filter).reduce(reduce)
    }
}
