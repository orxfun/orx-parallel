use super::{
    collect_into::par_collect_into::{merge_bag_and_pos_len, ParCollectInto},
    par_fmap::ParFMap,
    reduce::Reduce,
};
use crate::{
    core::{
        fmap_fil_cnt::fmap_fil_cnt,
        fmap_fil_col::{par_fmap_fil_col, seq_fmap_fil_col},
        fmap_fil_colx::{par_fmap_fil_colx, seq_fmap_fil_colx},
        fmap_fil_find::fmap_fil_find,
        fmap_fil_red::fmap_fil_red,
    },
    ParIter, ParMap, Params,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{ConIterOfVec, ConcurrentIter, IntoConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;

/// An iterator that maps the elements of the iterator with a given map function.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`ParMap::num_threads`] and [`ParMap::chunk_size`] methods.
pub struct ParFMapFilter<I, O, OI, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync,
    F: Fn(&O) -> bool + Send + Sync,
{
    iter: I,
    params: Params,
    fmap: M,
    filter: F,
}

impl<I, O, OI, M, F> ParIter for ParFMapFilter<I, O, OI, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter. Default is temporary also
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

    fn map<O2, M2>(self, map: M2) -> ParMap<ConIterOfVec<O>, O2, M2>
    where
        O2: Send + Sync + Default,
        M2: Fn(Self::Item) -> O2 + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParMap::new(iter, params, map)
    }

    fn flat_map<O2, OI2, FM>(self, fmap: FM) -> ParFMap<ConIterOfVec<O>, OI2::Item, OI2, FM>
    where
        O2: Send + Sync + Default,
        OI2: IntoIterator<Item = O2>,
        FM: Fn(Self::Item) -> OI2 + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParFMap::new(iter, params, fmap)
    }

    fn filter<F2>(self, filter: F2) -> ParFMapFilter<I, O, OI, M, impl Fn(&O) -> bool + Send + Sync>
    where
        F2: Fn(&Self::Item) -> bool + Send + Sync,
    {
        let (params, iter, fmap, filter1) = (self.params, self.iter, self.fmap, self.filter);
        let composed = move |x: &O| filter1(x) && filter(x);
        ParFMapFilter::new(iter, params, fmap, composed)
    }

    fn count(self) -> usize {
        let (params, iter, fmap, filter) = (self.params, self.iter, self.fmap, self.filter);
        fmap_fil_cnt(params, iter, fmap, filter)
    }

    // find
    fn find<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        let (params, iter, fmap, filter) = (self.params, self.iter, self.fmap, self.filter);
        let composed = move |x: &O| filter(x) && predicate(x);
        fmap_fil_find(params, iter, fmap, composed)
    }

    fn first(self) -> Option<Self::Item> {
        let (params, iter, fmap, filter) = (self.params, self.iter, self.fmap, self.filter);
        fmap_fil_find(params, iter, fmap, filter)
    }

    // collect

    fn collect_vec(self) -> Vec<Self::Item> {
        let mut vec = vec![];
        self.collect_bag(|x| vec.push(x));
        vec
    }

    fn collect(self) -> SplitVec<Self::Item> {
        let mut vec = SplitVec::new();
        self.collect_bag(|x| vec.push(x));
        vec
    }

    fn collect_into<C: ParCollectInto<Self::Item>>(self, output: C) -> C {
        output.fmap_filter_into(self)
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

impl<I, O, OI, M, F> ParFMapFilter<I, O, OI, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync,
    F: Fn(&O) -> bool + Send + Sync,
{
    // define

    pub(crate) fn new(iter: I, params: Params, fmap: M, filter: F) -> Self {
        Self {
            iter,
            params,
            fmap,
            filter,
        }
    }

    pub(crate) fn iter_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }

    // collect

    pub(crate) fn collect_bag<Push>(self, mut push: Push)
    where
        O: Default,
        Push: FnMut(O),
    {
        let (params, iter, fmap, filter) = (self.params, self.iter, self.fmap, self.filter);

        match params.is_sequential() {
            true => seq_fmap_fil_col(iter, fmap, filter, push),
            _ => {
                let bag = ConcurrentBag::new();
                let positions = ConcurrentOrderedBag::new();
                let (bag, pos_len) = par_fmap_fil_col(params, iter, fmap, filter, bag, positions);
                merge_bag_and_pos_len(bag, &pos_len, &mut push);
            }
        }
    }

    pub(crate) fn collect_bag_x<P>(self, collected: ConcurrentBag<O, P>) -> ConcurrentBag<O, P>
    where
        O: Default,
        P: PinnedVec<O>,
    {
        let (params, iter, fmap, filter) = (self.params, self.iter, self.fmap, self.filter);
        match params.is_sequential() {
            true => seq_fmap_fil_colx(iter, fmap, filter, collected),
            false => par_fmap_fil_colx(params, iter, fmap, filter, collected),
        }
    }
}

impl<I, O, OI, M, F> Reduce<O> for ParFMapFilter<I, O, OI, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync,
    OI: IntoIterator<Item = O>,
    M: Fn(I::Item) -> OI + Send + Sync,
    F: Fn(&O) -> bool + Send + Sync,
{
    fn reduce<R>(self, reduce: R) -> Option<O>
    where
        R: Fn(O, O) -> O + Send + Sync,
    {
        fmap_fil_red(self.params, self.iter, self.fmap, self.filter, reduce)
    }
}
