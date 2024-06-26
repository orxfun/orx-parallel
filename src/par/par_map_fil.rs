use super::{par_filtermap::ParFilterMap, par_flatmap::ParFlatMap, reduce::Reduce};
use crate::{
    core::{
        map_fil_cnt::map_fil_cnt,
        map_fil_col::{par_map_fil_col, seq_map_fil_col},
        map_fil_find::map_fil_find,
        map_fil_red::map_fil_red,
    },
    fn_sync::FnSync,
    par::collect_into::collect_into_core::merge_bag_and_positions,
    Fallible, ParCollectInto, ParIter, Params,
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
pub struct ParMapFilter<I, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    M: Fn(I::Item) -> O + FnSync,
    F: Fn(&O) -> bool + FnSync,
{
    iter: I,
    params: Params,
    map: M,
    filter: F,
}

impl<I, O, M, F> ParMapFilter<I, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    M: Fn(I::Item) -> O + FnSync,
    F: Fn(&O) -> bool + FnSync,
{
    pub(crate) fn new(iter: I, params: Params, map: M, filter: F) -> Self {
        Self {
            iter,
            params,
            map,
            filter,
        }
    }

    fn destruct(self) -> (Params, I, M, F) {
        (self.params, self.iter, self.map, self.filter)
    }

    // collect

    pub(crate) fn collect_bag_par<Output, NewOutput>(self, new_output: NewOutput) -> Output
    where
        Output: PinnedVec<O> + Debug,
        NewOutput: FnOnce(usize) -> Output,
    {
        debug_assert!(!self.params.is_sequential());

        let (params, iter, map, filter) = self.destruct();

        let bag = ConcurrentBag::new();
        match iter.try_get_len() {
            Some(len) => {
                let positions = ConcurrentOrderedBag::with_fixed_capacity(len);
                let (bag, positions) = par_map_fil_col(params, iter, map, filter, bag, positions);
                let mut output = new_output(bag.len());
                merge_bag_and_positions(bag, &positions, &mut output);
                output
            }
            None => {
                let positions = ConcurrentOrderedBag::new();
                let (bag, positions) = par_map_fil_col(params, iter, map, filter, bag, positions);
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

        let (_, iter, map, filter) = self.destruct();
        seq_map_fil_col(iter, map, filter, &mut output, push);
        output
    }

    // find

    /// Returns the first element of the iterator; returns None if the iterator is empty.
    ///
    /// If an element is found, the output is the tuple of:
    /// * the index of the element in the original collection, and
    /// * the value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// fn firstfac(x: usize) -> usize {
    ///     if x % 2 == 0 {
    ///         return 2;
    ///     };
    ///     for n in (1..).map(|m| 2 * m + 1).take_while(|m| m * m <= x) {
    ///         if x % n == 0 {
    ///             return n;
    ///         };
    ///     }
    ///     x
    /// }
    ///
    /// fn is_prime(n: &usize) -> bool {
    ///     match n {
    ///         0 | 1 => false,
    ///         _ => firstfac(*n) == *n,
    ///     }
    /// }
    ///
    /// let first_prime = (21..100).into_par().filter(is_prime).first_with_index();
    /// assert_eq!(first_prime, Some((2, 23)));
    ///
    /// let first_prime = (24..28).into_par().filter(is_prime).first_with_index();
    /// assert_eq!(first_prime, None);
    /// ```
    pub fn first_with_index(self) -> Option<(usize, O)> {
        let (params, iter, map, filter) = self.destruct();
        map_fil_find(params, iter, map, filter)
    }

    /// Returns the first element of the iterator satisfying the given `predicate`; returns None if the iterator is empty.
    ///
    /// If an element is found, the output is the tuple of:
    /// * the index of the element in the original collection, and
    /// * the value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// fn firstfac(x: usize) -> usize {
    ///     if x % 2 == 0 {
    ///         return 2;
    ///     };
    ///     for n in (1..).map(|m| 2 * m + 1).take_while(|m| m * m <= x) {
    ///         if x % n == 0 {
    ///             return n;
    ///         };
    ///     }
    ///     x
    /// }
    ///
    /// fn is_prime(n: &usize) -> bool {
    ///     match n {
    ///         0 | 1 => false,
    ///         _ => firstfac(*n) == *n,
    ///     }
    /// }
    ///
    /// let first_prime = (21..100).into_par().find_with_index(is_prime);
    /// assert_eq!(first_prime, Some((2, 23)));
    ///
    /// let first_prime = (24..28).into_par().find_with_index(is_prime);
    /// assert_eq!(first_prime, None);
    /// ```
    pub fn find_with_index<P>(self, predicate: P) -> Option<(usize, O)>
    where
        P: Fn(&O) -> bool + Send + Sync,
    {
        let (params, iter, map, filter) = self.destruct();
        let composed = move |x: &O| filter(x) && predicate(x);
        map_fil_find(params, iter, map, composed)
    }
}

impl<I, O, M, F> ParIter for ParMapFilter<I, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    M: Fn(I::Item) -> O + FnSync,
    F: Fn(&O) -> bool + FnSync,
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

    fn map<O2, M2>(
        self,
        map: M2,
    ) -> ParFilterMap<I, Option<O2>, O2, impl Fn(<I as ConcurrentIter>::Item) -> Option<O2> + FnSync>
    where
        O2: Send + Sync + Debug,
        M2: Fn(Self::Item) -> O2 + FnSync,
    {
        let (params, iter, map1, filter) = self.destruct();

        let composed_filter_map = move |x: I::Item| {
            let value = map1(x);
            match filter(&value) {
                false => None,
                true => Some(map(value)),
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

    fn filter<F2>(self, filter: F2) -> ParMapFilter<I, O, M, impl Fn(&O) -> bool + FnSync>
    where
        F2: Fn(&Self::Item) -> bool + FnSync,
    {
        let (params, iter, map, filter1) = self.destruct();
        let composed = move |x: &O| filter1(x) && filter(x);
        ParMapFilter::new(iter, params, map, composed)
    }

    fn filter_map<O2, FO, FM>(
        self,
        filter_map: FM,
    ) -> ParFilterMap<I, Option<O2>, O2, impl Fn(<I as ConcurrentIter>::Item) -> Option<O2> + FnSync>
    where
        O2: Send + Sync + Debug,
        FO: Fallible<O2> + Send + Sync + Debug,
        FM: Fn(Self::Item) -> FO + FnSync,
    {
        let (params, iter, map, filter) = self.destruct();
        let composed = move |x| {
            let mapped = map(x);
            match filter(&mapped) {
                false => None,
                true => filter_map(mapped).into_option(),
            }
        };
        ParFilterMap::new(iter, params, composed)
    }

    // reduce

    fn count(self) -> usize {
        let (params, iter, map, filter) = self.destruct();
        map_fil_cnt(params, iter, map, filter)
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
        output.map_filter_into(self)
    }
}

impl<I, O, M, F> Reduce<O> for ParMapFilter<I, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Debug,
    M: Fn(I::Item) -> O + FnSync,
    F: Fn(&O) -> bool + FnSync,
{
    fn reduce<R>(self, reduce: R) -> Option<O>
    where
        R: Fn(O, O) -> O + Send + Sync,
    {
        map_fil_red(self.params, self.iter, self.map, self.filter, reduce)
    }
}
