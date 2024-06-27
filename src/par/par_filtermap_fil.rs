use super::{
    par_filtermap::ParFilterMap, par_flatmap::ParFlatMap, par_map::ParMap, reduce::Reduce,
};
use crate::{
    core::{
        filtermap_fil_cnt::filtermap_fil_cnt,
        filtermap_fil_col::{par_filtermap_fil_col, seq_filtermap_fil_col},
        filtermap_fil_find::filtermap_fil_find,
        filtermap_fil_red::filtermap_fil_red,
    },
    par::collect_into::collect_into_core::merge_bag_and_positions,
    ChunkSize, Fallible, NumThreads, ParCollectInto, ParIter, Params,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{ConcurrentIter, IntoConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;
use std::{fmt::Debug, marker::PhantomData};

pub struct ParFilterMapFilter<I, FO, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    FO: Fallible<O> + Send + Sync + Debug,
    M: Fn(I::Item) -> FO + Send + Sync + Clone,
    F: Fn(&O) -> bool + Send + Sync + Clone,
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
    M: Fn(I::Item) -> FO + Send + Sync + Clone,
    F: Fn(&O) -> bool + Send + Sync + Clone,
{
    // define

    pub(crate) fn new(iter: I, params: Params, filter_map: M, filter: F) -> Self {
        Self {
            iter,
            params,
            filter_map,
            filter,
            phantom: PhantomData,
        }
    }

    // collect

    pub(crate) fn collect_bag_par<Output, NewOutput>(self, new_output: NewOutput) -> Output
    where
        Output: PinnedVec<O> + Debug,
        NewOutput: FnOnce(usize) -> Output,
    {
        debug_assert!(!self.params.is_sequential());

        let (params, iter, map, filter) = (self.params, self.iter, self.filter_map, self.filter);

        let bag = ConcurrentBag::new();
        match iter.try_get_len() {
            Some(len) => {
                let positions = ConcurrentOrderedBag::with_fixed_capacity(len);
                let (bag, positions) =
                    par_filtermap_fil_col(params, iter, map, filter, bag, positions);
                let mut output = new_output(bag.len());
                merge_bag_and_positions(bag, &positions, &mut output);
                output
            }
            None => {
                let positions = ConcurrentOrderedBag::new();
                let (bag, positions) =
                    par_filtermap_fil_col(params, iter, map, filter, bag, positions);
                let mut output = new_output(bag.len());
                merge_bag_and_positions(bag, &positions, &mut output);
                output
            }
        }
    }

    pub(crate) fn collect_bag_seq<Output, Push>(self, mut output: Output, push: Push) -> Output
    where
        Push: FnMut(&mut Output, O),
    {
        debug_assert!(self.params.is_sequential());

        let (iter, map, filter) = (self.iter, self.filter_map, self.filter);
        seq_filtermap_fil_col(iter, map, filter, &mut output, push);
        output
    }

    // find

    pub fn first_with_index(self) -> Option<(usize, O)> {
        let (params, iter, map, filter) = (self.params, self.iter, self.filter_map, self.filter);
        filtermap_fil_find(params, iter, map, filter)
    }

    pub fn find_with_index<P>(self, predicate: P) -> Option<(usize, O)>
    where
        P: Fn(&O) -> bool + Send + Sync,
    {
        let (params, iter, map, filter) = (self.params, self.iter, self.filter_map, self.filter);
        let composed = move |x: &O| filter(x) && predicate(x);
        filtermap_fil_find(params, iter, map, composed)
    }
}

impl<I, FO, O, M, F> ParIter for ParFilterMapFilter<I, FO, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    FO: Fallible<O> + Send + Sync + Debug,
    M: Fn(I::Item) -> FO + Send + Sync + Clone,
    F: Fn(&O) -> bool + Send + Sync + Clone,
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

    fn map<O2, M2>(self, map: M2) -> impl ParIter<Item = O2>
    where
        O2: Send + Sync + Debug,
        M2: Fn(Self::Item) -> O2 + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParMap::new(iter, params, map)
    }

    fn flat_map<O2, OI, FM>(self, flat_map: FM) -> impl ParIter<Item = O2>
    where
        O2: Send + Sync + Debug,
        OI: IntoIterator<Item = O2>,
        FM: Fn(Self::Item) -> OI + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParFlatMap::new(iter, params, flat_map)
    }

    fn filter<F2>(self, filter: F2) -> impl ParIter<Item = Self::Item>
    where
        F2: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        let (params, iter, map, filter1) = (self.params, self.iter, self.filter_map, self.filter);
        let composed = move |x: &O| filter1(x) && filter(x);
        ParFilterMapFilter::new(iter, params, map, composed)
    }

    fn filter_map<O2, FO2, FM>(
        self,
        filter_map: FM,
    ) -> ParFilterMap<
        I,
        Option<O2>,
        O2,
        impl Fn(<I as ConcurrentIter>::Item) -> Option<O2> + Send + Sync + Clone,
    >
    where
        O2: Send + Sync + Debug,
        FO2: Fallible<O2> + Send + Sync + Debug,
        FM: Fn(Self::Item) -> FO2 + Send + Sync + Clone,
    {
        let (params, iter, filter_map1, filter) =
            (self.params, self.iter, self.filter_map, self.filter);
        let composed = move |x| {
            let mapped = filter_map1(x);
            match mapped.has_value() {
                false => None,
                true => {
                    let value = mapped.unwrap();
                    match filter(&value) {
                        false => None,
                        true => filter_map(value).into_option(),
                    }
                }
            }
        };
        ParFilterMap::new(iter, params, composed)
    }

    // reduce

    fn count(self) -> usize {
        let (params, iter, map, filter) = (self.params, self.iter, self.filter_map, self.filter);
        filtermap_fil_cnt(params, iter, map, filter)
    }

    // find

    fn find<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        self.find_with_index(predicate).map(|x| x.1)
    }

    fn first(self) -> Option<Self::Item> {
        self.first_with_index().map(|x| x.1)
    }

    // collect

    fn collect_vec(self) -> Vec<Self::Item> {
        match self.params.is_sequential() {
            true => self.collect_bag_seq(Vec::new(), |v, x| v.push(x)),
            false => self.collect_bag_par(|len| FixedVec::new(len)).into(),
        }
    }

    fn collect(self) -> SplitVec<Self::Item> {
        match self.params.is_sequential() {
            true => self.collect_bag_seq(SplitVec::new(), |v, x| v.push(x)),
            false => {
                let new_output = |len| {
                    let mut vec = SplitVec::new();
                    unsafe { _ = vec.grow_to(len, false) };
                    vec
                };
                self.collect_bag_par(new_output)
            }
        }
    }

    fn collect_into<C: ParCollectInto<Self::Item>>(self, output: C) -> C {
        output.filtermap_filter_into(self)
    }
}

impl<I, FO, O, M, F> Reduce<O> for ParFilterMapFilter<I, FO, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    FO: Fallible<O> + Send + Sync + Debug,
    M: Fn(I::Item) -> FO + Send + Sync + Clone,
    F: Fn(&O) -> bool + Send + Sync + Clone,
{
    fn reduce<R>(self, reduce: R) -> Option<O>
    where
        R: Fn(O, O) -> O + Send + Sync,
    {
        filtermap_fil_red(self.params, self.iter, self.filter_map, self.filter, reduce)
    }
}
