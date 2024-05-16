use super::{
    collect_into::par_map_collect_into::ParMapCollectInto, par_map_fil::ParMapFilter,
    reduce::Reduce,
};
use crate::core::{
    default_fns::no_filter,
    map_col::map_col,
    map_fil_cnt::map_fil_cnt,
    map_fil_find::map_fil_find,
    map_fil_red::map_fil_red,
    params::{Params, RunParams},
};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;

pub struct ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync,
{
    iter: I,
    params: Params,
    map: M,
}

impl<I, O, M> ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync,
{
    // define

    pub(crate) fn new(iter: I, params: Params, map: M) -> Self {
        Self { iter, params, map }
    }

    pub(crate) fn iter_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }

    pub fn params(&self) -> Params {
        self.params
    }

    // helpers

    fn run_params(&self) -> RunParams {
        self.params.run_params(self.iter.try_get_len())
    }

    // transform

    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    pub fn chunk_size(mut self, chunk_size: usize) -> Self {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    pub fn map<M2, O2>(self, map: M2) -> ParMap<I, O2, impl Fn(I::Item) -> O2 + Send + Sync>
    where
        O2: Send + Sync,
        M2: Fn(O) -> O2 + Send + Sync,
    {
        let (params, iter, map1) = (self.params, self.iter, self.map);
        let composed = move |x| map(map1(x));
        ParMap::new(iter, params, composed)
    }

    pub fn filter<F>(self, filter: F) -> ParMapFilter<I, O, M, F>
    where
        F: Fn(&O) -> bool + Send + Sync,
    {
        ParMapFilter::new(self.iter, self.params, self.map, filter)
    }

    // collect

    pub(crate) fn collect_bag<P: PinnedVec<O>>(self, collected: ConcurrentOrderedBag<O, P>) -> P {
        map_col(self.run_params(), self.iter, self.map, collected)
    }

    pub fn collect_vec(self) -> Vec<O>
    where
        O: Default,
    {
        match self.iter_len() {
            Some(len) => self
                .collect_bag(ConcurrentOrderedBag::with_fixed_capacity(len))
                .into(),
            None => self.collect_bag(ConcurrentOrderedBag::new()).into(),
        }
    }

    pub fn collect(self) -> SplitVec<O> {
        self.collect_bag(ConcurrentOrderedBag::new())
    }

    // PANICS!
    pub fn collect_into<C: ParMapCollectInto<O>>(self, output: C) -> C {
        output.map_into(self)
    }

    // count
    pub fn count(self) -> usize {
        let (params, iter, map) = (self.run_params(), self.iter, self.map);
        map_fil_cnt(params, iter, map, no_filter)
    }

    // find
    pub fn next(self) -> Option<(usize, O)> {
        let (params, iter, map) = (self.run_params(), self.iter, self.map);
        map_fil_find(params, iter, map, no_filter)
    }

    pub fn find<P>(self, predicate: P) -> Option<(usize, O)>
    where
        P: Fn(&O) -> bool + Send + Sync,
    {
        let (params, iter, map) = (self.run_params(), self.iter, self.map);
        map_fil_find(params, iter, map, predicate)
    }
}

impl<I, O, M> Reduce<O> for ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync,
{
    fn reduce<R>(self, reduce: R) -> Option<O>
    where
        R: Fn(O, O) -> O + Send + Sync,
    {
        map_fil_red(self.run_params(), self.iter, self.map, no_filter, reduce)
    }
}
