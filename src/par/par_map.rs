use super::{
    collect_into::collect_into_core::ParCollectIntoCore, par_filtermap::ParFilterMap,
    par_flatmap::ParFlatMap, par_map_fil::ParMapFilter,
};
use crate::{
    core::{
        default_fns::no_filter, map_fil_cnt::map_fil_cnt, map_fil_find::map_fil_find,
        map_fil_red::map_fil_red,
    },
    par_iter::ParIter,
    Fallible, ParCollectInto, Params,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Recursive, SplitVec};

/// A parallel iterator.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`crate::ParIter::num_threads`] and [`crate::ParIter::chunk_size`] methods.
pub struct ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    iter: I,
    params: Params,
    map: M,
}

impl<I, O, M> ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    pub(crate) fn new(iter: I, params: Params, map: M) -> Self {
        Self { iter, params, map }
    }

    pub(crate) fn destruct(self) -> (Params, I, M) {
        (self.params, self.iter, self.map)
    }

    pub(crate) fn iter_len(&self) -> Option<usize> {
        self.iter.try_get_len()
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
        let (params, iter, map) = self.destruct();
        map_fil_find(params, iter, map, no_filter)
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
        P: Fn(&O) -> bool + Send + Sync + Clone,
    {
        let (params, iter, map) = self.destruct();
        map_fil_find(params, iter, map, predicate)
    }
}

impl<I, O, M> ParIter for ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
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

    fn map<O2, M2>(self, map: M2) -> ParMap<I, O2, impl Fn(I::Item) -> O2 + Send + Sync + Clone>
    where
        O2: Send + Sync,
        M2: Fn(Self::Item) -> O2 + Send + Sync + Clone,
    {
        let (params, iter, map1) = self.destruct();
        let composed = move |x| map(map1(x));
        ParMap::new(iter, params, composed)
    }

    fn flat_map<O2, OI, FM>(
        self,
        flat_map: FM,
    ) -> ParFlatMap<I, O2, OI, impl Fn(<I as ConcurrentIter>::Item) -> OI + Clone>
    where
        O2: Send + Sync,
        OI: IntoIterator<Item = O2>,
        FM: Fn(Self::Item) -> OI + Send + Sync + Clone,
    {
        let (params, iter, map) = self.destruct();
        let composed = move |x| flat_map(map(x));
        ParFlatMap::new(iter, params, composed)
    }

    fn filter<F>(self, filter: F) -> ParMapFilter<I, O, M, F>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        ParMapFilter::new(self.iter, self.params, self.map, filter)
    }

    fn filter_map<O2, FO, FM>(
        self,
        filter_map: FM,
    ) -> ParFilterMap<I, FO, O2, impl Fn(<I as ConcurrentIter>::Item) -> FO + Send + Sync + Clone>
    where
        O2: Send + Sync,
        FO: Fallible<O2> + Send + Sync,
        FM: Fn(Self::Item) -> FO + Send + Sync + Clone,
    {
        let (params, iter, map) = self.destruct();
        let composed = move |x| filter_map(map(x));
        ParFilterMap::new(iter, params, composed)
    }

    // reduce

    fn reduce<R>(self, reduce: R) -> Option<Self::Item>
    where
        R: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync + Clone,
    {
        map_fil_red(self.params, self.iter, self.map, no_filter, reduce)
    }

    fn count(self) -> usize {
        let (params, iter, map) = self.destruct();
        map_fil_cnt(params, iter, map, no_filter)
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
        Vec::new().map_into(self)
    }

    fn collect(self) -> SplitVec<Self::Item> {
        SplitVec::with_doubling_growth_and_fragments_capacity(32).map_into(self)
    }

    fn collect_into<C: ParCollectInto<Self::Item>>(self, output: C) -> C {
        output.map_into(self)
    }

    /// TODO: define the advantage!
    fn collect_x(self) -> SplitVec<Self::Item, Recursive> {
        self.filter(no_filter).collect_x()
    }
}
