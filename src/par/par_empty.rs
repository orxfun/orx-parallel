use super::{
    par_fil::ParFilter, par_filtermap::ParFilterMap, par_flatmap::ParFlatMap, par_map::ParMap,
};
use crate::{
    core::{
        default_fns::{map_self, no_filter},
        map_fil_cnt::map_fil_cnt,
        map_fil_find::map_fil_find,
        map_fil_red::map_fil_red,
    },
    par_iter::Par,
    ChunkSize, Fallible, NumThreads, ParCollectInto, Params,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Growth, SplitVec};

/// A parallel iterator.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`crate::Par::num_threads`] and [`crate::Par::chunk_size`] methods.
pub struct ParEmpty<I>
where
    I: ConcurrentIter,
{
    iter: I,
    params: Params,
}

impl<I> Par for ParEmpty<I>
where
    I: ConcurrentIter,
{
    type Item = I::Item;

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

    fn map<O, M>(self, map: M) -> ParMap<I, O, M>
    where
        O: Send + Sync,
        M: Fn(Self::Item) -> O + Send + Sync + Clone,
    {
        ParMap::new(self.iter, self.params, map)
    }

    fn flat_map<O, OI, FM>(self, flat_map: FM) -> ParFlatMap<I, O, OI, FM>
    where
        O: Send + Sync,
        OI: IntoIterator<Item = O>,
        FM: Fn(Self::Item) -> OI + Send + Sync + Clone,
    {
        ParFlatMap::new(self.iter, self.params, flat_map)
    }

    fn filter<F>(self, filter: F) -> ParFilter<I, F>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        ParFilter::new(self.iter, self.params, filter)
    }

    fn filter_map<O, FO, FM>(self, filter_map: FM) -> ParFilterMap<I, FO, O, FM>
    where
        O: Send + Sync,
        FO: Fallible<O> + Send + Sync,
        FM: Fn(Self::Item) -> FO + Send + Sync + Clone,
    {
        ParFilterMap::new(self.iter, self.params, filter_map)
    }

    // reduce

    fn reduce<R>(self, reduce: R) -> Option<Self::Item>
    where
        R: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync + Clone,
    {
        let (params, iter) = (self.params, self.iter.into_con_iter_x());
        map_fil_red(params, iter, map_self, no_filter, reduce)
    }

    fn count(self) -> usize {
        let (params, iter) = (self.params, self.iter.into_con_iter_x());
        map_fil_cnt(params, iter, map_self, no_filter)
    }

    // find

    fn find<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + Send + Sync,
    {
        self.find_with_index(predicate).map(|x| x.1)
    }

    fn first(self) -> Option<Self::Item> {
        self.first_with_index().map(|x| x.1)
    }

    // collect

    fn collect_vec(self) -> Vec<Self::Item> {
        self.iter.into_seq_iter().collect()
    }

    fn collect(self) -> SplitVec<Self::Item> {
        self.iter.into_seq_iter().collect()
    }

    fn collect_into<C: ParCollectInto<Self::Item>>(self, output: C) -> C {
        output.seq_extend(self.iter.into_seq_iter())
    }

    fn collect_x(self) -> SplitVec<Self::Item, impl Growth> {
        self.collect()
    }
}

impl<I> ParEmpty<I>
where
    I: ConcurrentIter,
{
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            params: Params::default(),
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
        let (params, iter) = (self.params, self.iter);
        map_fil_find(params, iter, map_self, no_filter)
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
        let (params, iter) = (self.params, self.iter);
        map_fil_find(params, iter, map_self, predicate)
    }
}
