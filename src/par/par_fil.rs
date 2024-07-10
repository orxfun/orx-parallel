use super::{par_filtermap::ParFilterMap, par_flatmap::ParFlatMap, par_map_fil::ParMapFilter};
use crate::{
    core::{
        default_fns::map_self, map_fil_cnt::map_fil_cnt, map_fil_find::map_fil_find,
        map_fil_red::map_fil_red,
    },
    Fallible, Par, ParCollectInto, Params,
};
use orx_concurrent_iter::{ConIterOfVec, ConcurrentIter, IntoConcurrentIter};
use orx_split_vec::SplitVec;

/// A parallel iterator.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`crate::Par::num_threads`] and [`crate::Par::chunk_size`] methods.
pub struct ParFilter<I, F>
where
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync + Clone,
{
    iter: I,
    params: Params,
    filter: F,
}

impl<I, F> Par for ParFilter<I, F>
where
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync + Clone,
{
    type Item = I::Item;

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

    fn map<O, M>(
        self,
        map: M,
    ) -> ParFilterMap<
        I,
        Option<O>,
        O,
        impl Fn(<I as ConcurrentIter>::Item) -> Option<O> + Send + Sync + Clone,
    >
    where
        O: Send + Sync,
        M: Fn(Self::Item) -> O + Send + Sync + Clone,
    {
        let (params, iter, filter) = (self.params, self.iter, self.filter);
        let composed_filter_map = move |x| match filter(&x) {
            false => None,
            true => Some(map(x)),
        };
        ParFilterMap::new(iter, params, composed_filter_map)
    }

    fn flat_map<O, OI, FM>(
        self,
        flat_map: FM,
    ) -> ParFlatMap<ConIterOfVec<<I as ConcurrentIter>::Item>, O, OI, FM>
    where
        O: Send + Sync,
        OI: IntoIterator<Item = O>,
        FM: Fn(Self::Item) -> OI + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParFlatMap::new(iter, params, flat_map)
    }

    fn filter<F2>(self, filter: F2) -> ParFilter<I, impl Fn(&I::Item) -> bool + Send + Sync + Clone>
    where
        F2: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        let (params, iter, filter1) = (self.params, self.iter, self.filter);
        let composed_filter = move |x: &I::Item| filter1(x) && filter(x);
        ParFilter::new(iter, params, composed_filter)
    }

    fn filter_map<O, FO, FM>(
        self,
        filter_map: FM,
    ) -> ParFilterMap<
        I,
        Option<O>,
        O,
        impl Fn(<I as ConcurrentIter>::Item) -> Option<O> + Send + Sync + Clone,
    >
    where
        O: Send + Sync,
        FO: Fallible<O> + Send + Sync,
        FM: Fn(Self::Item) -> FO + Send + Sync + Clone,
    {
        let (params, iter, filter) = (self.params, self.iter, self.filter);
        let composed_filter_map = move |x| match filter(&x) {
            false => None,
            true => filter_map(x).into_option(),
        };
        ParFilterMap::new(iter, params, composed_filter_map)
    }

    // reduce

    fn reduce<R>(self, reduce: R) -> Option<Self::Item>
    where
        R: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync + Clone,
    {
        map_fil_red(self.params, self.iter, map_self, self.filter, reduce)
    }

    fn count(self) -> usize {
        let (params, iter, filter) = (self.params, self.iter, self.filter);
        map_fil_cnt(params, iter, map_self, filter)
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
        ParMapFilter::new(self.iter, self.params, map_self, self.filter).collect_vec()
    }

    fn collect(self) -> SplitVec<Self::Item> {
        ParMapFilter::new(self.iter, self.params, map_self, self.filter).collect()
    }

    fn collect_into<C: ParCollectInto<Self::Item>>(self, output: C) -> C {
        ParMapFilter::new(self.iter, self.params, map_self, self.filter).collect_into(output)
    }
}

impl<I, F> ParFilter<I, F>
where
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync + Clone,
{
    pub(crate) fn new(iter: I, params: Params, filter: F) -> Self {
        Self {
            iter,
            params,
            filter,
        }
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
    pub fn first_with_index(self) -> Option<(usize, I::Item)> {
        let (params, iter, filter) = (self.params, self.iter, self.filter);
        map_fil_find(params, iter, map_self, filter)
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
    pub fn find_with_index<P>(self, predicate: P) -> Option<(usize, I::Item)>
    where
        P: Fn(&I::Item) -> bool + Send + Sync,
    {
        let (params, iter, filter) = (self.params, self.iter, self.filter);
        let composed = move |x: &I::Item| filter(x) && predicate(x);
        map_fil_find(params, iter, map_self, composed)
    }
}
