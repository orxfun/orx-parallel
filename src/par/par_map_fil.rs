use super::{
    collect_into::par_map_fil_collect_into::{merge_bag_and_positions, ParMapFilterCollectInto},
    par_map::ParMap,
    reduce::Reduce,
};
use crate::core::{
    map_fil_cnt::map_fil_cnt,
    map_fil_col::map_fil_col,
    map_fil_find::map_fil_find,
    map_fil_red::map_fil_red,
    params::{Params, RunParams},
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{ConcurrentIter, IntoConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;

pub struct ParMapFilter<I, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync,
    F: Fn(&O) -> bool + Send + Sync,
{
    iter: I,
    params: Params,
    map: M,
    filter: F,
}

impl<I, O, M, F> ParMapFilter<I, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync,
    F: Fn(&O) -> bool + Send + Sync,
{
    // define

    pub(crate) fn new(iter: I, params: Params, map: M, filter: F) -> Self {
        Self {
            iter,
            params,
            map,
            filter,
        }
    }

    pub(crate) fn iter_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }

    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    pub fn chunk_size(mut self, chunk_size: usize) -> Self {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    pub fn params(&self) -> Params {
        self.params
    }

    // helpers

    fn run_params(&self) -> RunParams {
        self.params.run_params(self.iter.try_get_len())
    }

    // transform

    pub fn map<O2, M2>(self, map: M2) -> ParMap<impl ConcurrentIter<Item = O>, O2, M2>
    where
        O2: Send + Sync,
        M2: Fn(O) -> O2 + Send + Sync,
        O: Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter. Default is temporary also
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParMap::new(iter, params, map)
    }

    pub fn filter<F2>(self, filter: F2) -> ParMapFilter<I, O, M, impl Fn(&O) -> bool + Send + Sync>
    where
        F2: Fn(&O) -> bool + Send + Sync,
    {
        let (params, iter, map, filter1) = (self.params, self.iter, self.map, self.filter);
        let composed = move |x: &O| filter1(x) && filter(x);
        ParMapFilter::new(iter, params, map, composed)
    }

    // collect

    pub(crate) fn collect_bag<Push>(self, mut push: Push)
    where
        O: Default,
        Push: FnMut(O),
    {
        let (params, iter, map, filter) = (self.run_params(), self.iter, self.map, self.filter);
        let bag = ConcurrentBag::new();
        match iter.try_get_len() {
            Some(len) => {
                let positions = ConcurrentOrderedBag::with_fixed_capacity(len);
                let (bag, positions) = map_fil_col(params, iter, map, filter, bag, positions);
                merge_bag_and_positions(bag, &positions, &mut push);
            }
            None => {
                let positions = ConcurrentOrderedBag::new();
                let (bag, positions) = map_fil_col(params, iter, map, filter, bag, positions);
                merge_bag_and_positions(bag, &positions, &mut push);
            }
        };
    }

    pub fn collect_vec(self) -> Vec<O>
    where
        O: Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter
    {
        let mut vec = vec![];
        self.collect_bag(|x| vec.push(x));
        vec
    }

    pub fn collect(self) -> SplitVec<O>
    where
        O: Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter
    {
        let mut vec = SplitVec::new();
        self.collect_bag(|x| vec.push(x));
        vec
    }

    pub fn collect_into<C: ParMapFilterCollectInto<O>>(self, output: C) -> C
    where
        O: Default,
    {
        output.map_filter_into(self)
    }

    // count
    pub fn count(self) -> usize {
        let (params, iter, map, filter) = (self.run_params(), self.iter, self.map, self.filter);
        map_fil_cnt(params, iter, map, filter)
    }

    // find
    pub fn next(self) -> Option<(usize, O)> {
        let (params, iter, map, filter) = (self.run_params(), self.iter, self.map, self.filter);
        map_fil_find(params, iter, map, filter)
    }

    pub fn find<P>(self, predicate: P) -> Option<(usize, O)>
    where
        P: Fn(&O) -> bool + Send + Sync,
    {
        let (params, iter, map, filter) = (self.run_params(), self.iter, self.map, self.filter);
        let composed = move |x: &O| filter(x) && predicate(x);
        map_fil_find(params, iter, map, composed)
    }
}

impl<I, O, M, F> Reduce<O> for ParMapFilter<I, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync,
    F: Fn(&O) -> bool + Send + Sync,
{
    fn reduce<R>(self, reduce: R) -> Option<O>
    where
        R: Fn(O, O) -> O + Send + Sync,
    {
        map_fil_red(self.run_params(), self.iter, self.map, self.filter, reduce)
    }
}
