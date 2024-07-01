use super::{
    par_filtermap::ParFilterMap, par_flatmap::ParFlatMap, par_map::ParMap, reduce::Reduce,
};
use crate::{
    core::{
        flatmap_fil_cnt::fmap_fil_cnt,
        flatmap_fil_col::{par_fmap_fil_col, seq_fmap_fil_col},
        flatmap_fil_colx::{par_fmap_fil_colx, seq_fmap_fil_colx},
        flatmap_fil_find::fmap_fil_find,
        flatmap_fil_red::fmap_fil_red,
    },
    fn_sync::FnSync,
    par::collect_into::collect_into_core::merge_bag_and_pos_len,
    ParCollectInto, ParIter, Params,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{ConIterOfVec, ConcurrentIter, IntoConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;
use std::fmt::Debug;

/// A parallel iterator.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`crate::ParIter::num_threads`] and [`crate::ParIter::chunk_size`] methods.
pub struct ParFlatMapFilter<I, O, OI, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
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
    O: Send + Sync + Debug,
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

    fn destruct(self) -> (Params, I, M, F) {
        (self.params, self.iter, self.flat_map, self.filter)
    }

    // collect

    pub(crate) fn collect_bag_par<Output, NewOutput>(self, new_output: NewOutput) -> Output
    where
        Output: PinnedVec<O> + Debug,
        NewOutput: FnOnce(usize) -> Output,
    {
        debug_assert!(!self.params.is_sequential());

        let (params, iter, flat_map, filter) = self.destruct();

        let bag = ConcurrentBag::new();
        let positions = ConcurrentOrderedBag::new();
        let (bag, pos_len) = par_fmap_fil_col(params, iter, flat_map, filter, bag, positions);
        let mut output = new_output(bag.len());
        merge_bag_and_pos_len(bag, &pos_len, &mut output);
        output
    }

    pub(crate) fn collect_bag_seq<Output, Push>(self, mut output: Output, push: Push) -> Output
    where
        Push: FnMut(&mut Output, O),
    {
        debug_assert!(self.params.is_sequential());

        let (_, iter, flat_map, filter) = self.destruct();

        seq_fmap_fil_col(iter, flat_map, filter, &mut output, push);
        output
    }

    pub(crate) fn collect_bag_x<P>(self, collected: ConcurrentBag<O, P>) -> ConcurrentBag<O, P>
    where
        P: PinnedVec<O>,
    {
        let (params, iter, flat_map, filter) = self.destruct();

        match params.is_sequential() {
            true => seq_fmap_fil_colx(iter, flat_map, filter, collected),
            false => par_fmap_fil_colx(params, iter, flat_map, filter, collected),
        }
    }
}

impl<I, O, OI, M, F> ParIter for ParFlatMapFilter<I, O, OI, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
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

    // transform

    fn map<O2, M2>(self, map: M2) -> ParMap<ConIterOfVec<O>, O2, M2>
    where
        O2: Send + Sync + Debug,
        M2: Fn(Self::Item) -> O2 + FnSync,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParMap::new(iter, params, map)
    }

    fn flat_map<O2, OI2, FM>(self, flat_map: FM) -> ParFlatMap<ConIterOfVec<O>, OI2::Item, OI2, FM>
    where
        O2: Send + Sync + Debug,
        OI2: IntoIterator<Item = O2>,
        FM: Fn(Self::Item) -> OI2 + FnSync,
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
        O2: Send + Sync + Debug,
        FO: crate::Fallible<O2> + Send + Sync + Debug,
        FM: Fn(Self::Item) -> FO + FnSync,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParFilterMap::new(iter, params, filter_map)
    }

    // reduce

    fn count(self) -> usize {
        let (params, iter, flat_map, filter) = self.destruct();
        fmap_fil_cnt(params, iter, flat_map, filter)
    }

    // find
    fn find<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + FnSync,
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
        output.flatmap_filter_into(self)
    }

    fn collect_x_vec(self) -> Vec<Self::Item> {
        self.collect_bag_x(ConcurrentBag::new()).into_inner().into()
    }

    fn collect_x(self) -> SplitVec<Self::Item> {
        self.collect_bag_x(ConcurrentBag::new()).into_inner()
    }

    fn collect_x_into<B: ParCollectInto<Self::Item>>(self, output: B) -> B {
        let x = self.collect_bag_x(output.into_concurrent_bag());
        B::from_concurrent_bag(x)
    }
}

impl<I, O, OI, M, F> Reduce<O> for ParFlatMapFilter<I, O, OI, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync,
    F: Fn(&O) -> bool + Send + Sync,
{
    fn reduce<R>(self, reduce: R) -> Option<O>
    where
        R: Fn(O, O) -> O + Send + Sync,
    {
        fmap_fil_red(self.params, self.iter, self.flat_map, self.filter, reduce)
    }
}
