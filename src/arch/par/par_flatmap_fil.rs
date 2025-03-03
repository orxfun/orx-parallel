use super::{
    collect_into::collect_into_core::ParCollectIntoCore, par_filtermap::ParFilterMap,
    par_flatmap::ParFlatMap, par_map::ParMap,
};
use crate::{
    core::{
        flatmap_fil_cnt::fmap_fil_cnt, flatmap_fil_col_x::par_flatmap_fil_col_x_rec,
        flatmap_fil_find::fmap_fil_find, flatmap_fil_red::fmap_fil_red,
    },
    Par, ParCollectInto, Params,
};
use orx_concurrent_iter::{ConIterOfVec, ConcurrentIter, ConcurrentIterX, IntoConcurrentIter};
use orx_split_vec::{Recursive, SplitVec};

/// A parallel iterator.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`crate::Par::num_threads`] and [`crate::Par::chunk_size`] methods.
pub struct ParFlatMapFilter<I, O, OI, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync,
    F: Fn(&O) -> bool + Send + Sync,
{
    iter: I,
    params: Params,
    flat_map: M,
    filter: F,
}

impl<I, O, OI, M, F> ParFlatMapFilter<I, O, OI, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync,
    F: Fn(&O) -> bool + Send + Sync,
{
    pub(crate) fn new(iter: I, params: Params, flat_map: M, filter: F) -> Self {
        Self {
            iter,
            params,
            flat_map,
            filter,
        }
    }

    pub(crate) fn destruct(self) -> (Params, I, M, F) {
        (self.params, self.iter, self.flat_map, self.filter)
    }

    pub(crate) fn destruct_x(self) -> (Params, impl ConcurrentIterX<Item = I::Item>, M, F) {
        (
            self.params,
            self.iter.into_con_iter_x(),
            self.flat_map,
            self.filter,
        )
    }
}

impl<I, O, OI, M, F> Par for ParFlatMapFilter<I, O, OI, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync,
    F: Fn(&O) -> bool + Send + Sync,
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

    // transformations

    fn map<O2, M2>(self, map: M2) -> ParMap<ConIterOfVec<O>, O2, M2>
    where
        O2: Send + Sync,
        M2: Fn(Self::Item) -> O2 + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParMap::new(iter, params, map)
    }

    fn flat_map<O2, OI2, FM>(self, flat_map: FM) -> ParFlatMap<ConIterOfVec<O>, OI2::Item, OI2, FM>
    where
        O2: Send + Sync,
        OI2: IntoIterator<Item = O2>,
        FM: Fn(Self::Item) -> OI2 + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParFlatMap::new(iter, params, flat_map)
    }

    fn filter<F2>(
        self,
        filter: F2,
    ) -> ParFlatMapFilter<I, O, OI, M, impl Fn(&O) -> bool + Send + Sync>
    where
        F2: Fn(&Self::Item) -> bool + Send + Sync,
    {
        let (params, iter, flat_map, filter1) = self.destruct();
        let composed = move |x: &O| filter1(x) && filter(x);
        ParFlatMapFilter::new(iter, params, flat_map, composed)
    }

    fn filter_map<O2, FO, FM>(self, filter_map: FM) -> ParFilterMap<ConIterOfVec<O>, FO, O2, FM>
    where
        O2: Send + Sync,
        FO: crate::Fallible<O2> + Send + Sync,
        FM: Fn(Self::Item) -> FO + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParFilterMap::new(iter, params, filter_map)
    }

    // reduce

    fn reduce<R>(self, reduce: R) -> Option<Self::Item>
    where
        R: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync + Clone,
    {
        let (params, iter, flat_map, filter) = self.destruct_x();
        fmap_fil_red(params, iter, flat_map, filter, reduce)
    }

    fn count(self) -> usize {
        let (params, iter, flat_map, filter) = self.destruct_x();
        fmap_fil_cnt(params, iter, flat_map, filter)
    }

    // find
    fn find<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        let (params, iter, flat_map, filter) = self.destruct();
        let composed = move |x: &O| filter(x) && predicate(x);
        fmap_fil_find(params, iter, flat_map, composed)
    }

    fn first(self) -> Option<Self::Item> {
        let (params, iter, flat_map, filter) = self.destruct();
        fmap_fil_find(params, iter, flat_map, filter)
    }

    // collect

    fn collect_vec(self) -> Vec<Self::Item> {
        Vec::new().flatmap_filter_into(self)
    }

    fn collect(self) -> SplitVec<Self::Item> {
        SplitVec::new().flatmap_filter_into(self)
    }

    fn collect_into<C: ParCollectInto<Self::Item>>(self, output: C) -> C {
        output.flatmap_filter_into(self)
    }

    /// TODO: define the advantage!
    fn collect_x(self) -> SplitVec<Self::Item, Recursive> {
        match self.params().is_sequential() {
            true => SplitVec::from(self.collect()),
            false => {
                let mut recursive = SplitVec::with_recursive_growth();
                let (params, iter, map, filter) = self.destruct_x();
                par_flatmap_fil_col_x_rec(params, iter, map, filter, &mut recursive);
                recursive
            }
        }
    }
}
