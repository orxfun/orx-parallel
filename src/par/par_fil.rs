use std::fmt::Debug;

use super::{
    collect_into::par_map_fil_collect_into::ParMapFilterCollectInto, par_map::ParMap,
    reduce::Reduce,
};
use crate::{
    core::{
        default_fns::map_self, map_fil_cnt::map_fil_cnt, map_fil_find::map_fil_find,
        map_fil_red::map_fil_red, params::Params, run_params::RunParams,
    },
    ParMapFilter,
};
use orx_concurrent_iter::{ConcurrentIter, IntoConcurrentIter};
use orx_split_vec::SplitVec;

pub struct ParFilter<I, F>
where
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync,
{
    iter: I,
    params: Params,
    filter: F,
}

impl<I, F> ParFilter<I, F>
where
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync,
{
    // define

    pub(crate) fn new(iter: I, params: Params, filter: F) -> Self {
        Self {
            iter,
            params,
            filter,
        }
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

    pub fn map<O, M>(self, map: M) -> ParMap<impl ConcurrentIter<Item = I::Item>, O, M>
    where
        O: Send + Sync,
        M: Fn(I::Item) -> O + Send + Sync,
        I::Item: Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter. Default is temporary also
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParMap::new(iter, params, map)
    }

    pub fn filter<F2>(self, filter: F2) -> ParFilter<I, impl Fn(&I::Item) -> bool + Send + Sync>
    where
        F2: Fn(&I::Item) -> bool + Send + Sync,
    {
        let (params, iter, filter1) = (self.params, self.iter, self.filter);
        let composed = move |x: &I::Item| filter1(x) && filter(x);
        ParFilter::new(iter, params, composed)
    }

    // collect

    pub fn collect_vec(self) -> Vec<I::Item>
    where
        I::Item: Default,
    {
        ParMapFilter::new(self.iter, self.params, map_self, self.filter).collect_vec()
    }

    pub fn collect(self) -> SplitVec<I::Item>
    where
        I::Item: Default,
    {
        ParMapFilter::new(self.iter, self.params, map_self, self.filter).collect()
    }

    // PANICS!
    pub fn collect_into<C: ParMapFilterCollectInto<I::Item>>(self, output: C) -> C
    where
        I::Item: Default,
    {
        ParMapFilter::new(self.iter, self.params, map_self, self.filter).collect_into(output)
    }

    // count
    pub fn count(self) -> usize {
        let (params, iter, filter) = (self.run_params(), self.iter, self.filter);
        map_fil_cnt(params, iter, map_self, filter)
    }

    // find
    pub fn next(self) -> Option<(usize, I::Item)> {
        let (params, iter, filter) = (self.run_params(), self.iter, self.filter);
        map_fil_find(params, iter, map_self, filter)
    }

    pub fn find<P>(self, predicate: P) -> Option<(usize, I::Item)>
    where
        P: Fn(&I::Item) -> bool + Send + Sync,
    {
        let (params, iter, filter) = (self.run_params(), self.iter, self.filter);
        let composed = move |x: &I::Item| filter(x) && predicate(x);
        map_fil_find(params, iter, map_self, composed)
    }
}

impl<I, F> Reduce<I::Item> for ParFilter<I, F>
where
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync,
    I::Item: Debug,
{
    fn reduce<R>(self, reduce: R) -> Option<I::Item>
    where
        R: Fn(I::Item, I::Item) -> I::Item + Send + Sync,
    {
        map_fil_red(self.run_params(), self.iter, map_self, self.filter, reduce)
    }
}
