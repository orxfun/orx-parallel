use super::{
    collect_into::collect_into_core::ParCollectIntoCore, par_filtermap::ParFilterMap,
    par_flatmap::ParFlatMap, reduce::Reduce,
};
use crate::{
    core::{
        filtermap_fil_cnt::filtermap_fil_cnt, filtermap_fil_col_x::par_filtermap_fil_col_x_rec,
        filtermap_fil_find::filtermap_fil_find, filtermap_fil_red::filtermap_fil_red,
    },
    fn_sync::FnSync,
    ChunkSize, Fallible, NumThreads, ParCollectInto, ParIter, Params,
};
use orx_concurrent_iter::{ConIterOfVec, ConcurrentIter, IntoConcurrentIter};
use orx_split_vec::{Recursive, SplitVec};
use std::{fmt::Debug, marker::PhantomData};

/// A parallel iterator.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`crate::ParIter::num_threads`] and [`crate::ParIter::chunk_size`] methods.
pub struct ParFilterMapFilter<I, FO, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    FO: Fallible<O> + Send + Sync + Debug,
    M: Fn(I::Item) -> FO + FnSync,
    F: Fn(&O) -> bool + FnSync,
{
    iter: I,
    params: Params,
    filter_map: M,
    filter: F,
    phantom: PhantomData<O>,
}

impl<I, FO, O, M, F> ParFilterMapFilter<I, FO, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    FO: Fallible<O> + Send + Sync + Debug,
    M: Fn(I::Item) -> FO + FnSync,
    F: Fn(&O) -> bool + FnSync,
{
    pub(crate) fn new(iter: I, params: Params, filter_map: M, filter: F) -> Self {
        Self {
            iter,
            params,
            filter_map,
            filter,
            phantom: PhantomData,
        }
    }

    pub(crate) fn destruct(self) -> (Params, I, M, F) {
        (self.params, self.iter, self.filter_map, self.filter)
    }

    // find

    pub fn first_with_index(self) -> Option<(usize, O)> {
        let (params, iter, filter_map, filter) = self.destruct();
        filtermap_fil_find(params, iter, filter_map, filter)
    }

    pub fn find_with_index<P>(self, predicate: P) -> Option<(usize, O)>
    where
        P: Fn(&O) -> bool + Send + Sync,
    {
        let (params, iter, filter_map, filter) = self.destruct();
        let composed = move |x: &O| filter(x) && predicate(x);
        filtermap_fil_find(params, iter, filter_map, composed)
    }
}

impl<I, FO, O, M, F> ParIter for ParFilterMapFilter<I, FO, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    FO: Fallible<O> + Send + Sync + Debug,
    M: Fn(I::Item) -> FO + FnSync,
    F: Fn(&O) -> bool + FnSync,
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
        let (params, iter, filter_map, filter) = self.destruct();

        let composed_filter_map = move |x: I::Item| {
            let maybe = filter_map(x);
            match maybe.has_value() {
                false => None,
                true => {
                    let value = maybe.value();
                    match filter(&value) {
                        false => None,
                        true => Some(map(value)),
                    }
                }
            }
        };

        ParFilterMap::new(iter, params, composed_filter_map)
    }

    fn flat_map<O2, OI, FM>(self, flat_map: FM) -> ParFlatMap<ConIterOfVec<O>, O2, OI, FM>
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

    fn filter<F2>(self, filter: F2) -> ParFilterMapFilter<I, FO, O, M, impl Fn(&O) -> bool + FnSync>
    where
        F2: Fn(&Self::Item) -> bool + FnSync,
    {
        let (params, iter, filter_map, filter1) = self.destruct();
        let composed_filter = move |x: &O| filter1(x) && filter(x);
        ParFilterMapFilter::new(iter, params, filter_map, composed_filter)
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
        let (params, iter, filter_map1, filter) = self.destruct();

        let composed_filter_map = move |x| {
            let mapped = filter_map1(x);
            match mapped.has_value() {
                false => None,
                true => {
                    let value = mapped.value();
                    match filter(&value) {
                        false => None,
                        true => filter_map(value).into_option(),
                    }
                }
            }
        };
        ParFilterMap::new(iter, params, composed_filter_map)
    }

    // reduce

    fn count(self) -> usize {
        let (params, iter, map, filter) = (self.params, self.iter, self.filter_map, self.filter);
        filtermap_fil_cnt(params, iter, map, filter)
    }

    // find

    fn find<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + FnSync,
    {
        self.find_with_index(predicate).map(|x| x.1)
    }

    fn first(self) -> Option<Self::Item> {
        self.first_with_index().map(|x| x.1)
    }

    // collect

    fn collect_vec(self) -> Vec<Self::Item> {
        Vec::new().filtermap_filter_into(self)
    }

    fn collect(self) -> SplitVec<Self::Item> {
        SplitVec::new().filtermap_filter_into(self)
    }

    fn collect_into<C: ParCollectInto<Self::Item>>(self, output: C) -> C {
        output.filtermap_filter_into(self)
    }

    fn collect_x(self) -> SplitVec<Self::Item, Recursive> {
        match self.params().is_sequential() {
            true => SplitVec::from(self.collect()),
            false => {
                let mut recursive = SplitVec::with_recursive_growth();
                let (params, iter, map, filter) = self.destruct();
                par_filtermap_fil_col_x_rec(params, iter, map, filter, &mut recursive);
                recursive
            }
        }
    }
}

impl<I, FO, O, M, F> Reduce<O> for ParFilterMapFilter<I, FO, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    FO: Fallible<O> + Send + Sync + Debug,
    M: Fn(I::Item) -> FO + FnSync,
    F: Fn(&O) -> bool + FnSync,
{
    fn reduce<R>(self, reduce: R) -> Option<O>
    where
        R: Fn(O, O) -> O + Send + Sync,
    {
        filtermap_fil_red(self.params, self.iter, self.filter_map, self.filter, reduce)
    }
}
