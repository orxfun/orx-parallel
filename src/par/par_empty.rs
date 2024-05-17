use std::fmt::Debug;

use super::{par_fil::ParFilter, par_map::ParMap, reduce::Reduce};
use crate::core::{
    default_fns::{map_self, no_filter},
    map_fil_cnt::map_fil_cnt,
    map_fil_find::map_fil_find,
    map_fil_red::map_fil_red,
    params::Params,
    run_params::RunParams,
};
use orx_concurrent_iter::ConcurrentIter;

pub struct Par<I>
where
    I: ConcurrentIter,
{
    iter: I,
    params: Params,
}

impl<I> Par<I>
where
    I: ConcurrentIter,
{
    // define

    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            params: Params::default(),
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

    pub fn map<O, M>(self, map: M) -> ParMap<I, O, M>
    where
        O: Send + Sync,
        M: Fn(I::Item) -> O + Send + Sync,
    {
        ParMap::new(self.iter, self.params, map)
    }

    pub fn filter<F>(self, filter: F) -> ParFilter<I, F>
    where
        F: Fn(&I::Item) -> bool + Send + Sync,
    {
        ParFilter::new(self.iter, self.params, filter)
    }

    // count

    pub fn count(self) -> usize {
        let (params, iter) = (self.run_params(), self.iter);
        map_fil_cnt(params, iter, map_self, no_filter)
    }

    // find

    pub fn next(self) -> Option<(usize, I::Item)> {
        let (params, iter) = (self.run_params(), self.iter);
        map_fil_find(params, iter, map_self, no_filter)
    }

    pub fn find<P>(self, predicate: P) -> Option<(usize, I::Item)>
    where
        P: Fn(&I::Item) -> bool + Send + Sync,
    {
        let (params, iter) = (self.run_params(), self.iter);
        map_fil_find(params, iter, map_self, predicate)
    }
}

impl<I> Reduce<I::Item> for Par<I>
where
    I: ConcurrentIter,
    I::Item: Debug,
{
    fn reduce<R>(self, reduce: R) -> Option<I::Item>
    where
        R: Fn(I::Item, I::Item) -> I::Item + Send + Sync,
    {
        map_fil_red(self.run_params(), self.iter, map_self, no_filter, reduce)
    }
}
