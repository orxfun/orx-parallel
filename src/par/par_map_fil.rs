use super::{
    collect_into::par_collect_into::{merge_bag_and_positions, ParCollectInto},
    par_fmap::ParFMap,
    par_map::ParMap,
    reduce::Reduce,
};
use crate::{
    core::{
        map_fil_cnt::map_fil_cnt,
        map_fil_col::{par_map_fil_col, seq_map_fil_col},
        map_fil_find::map_fil_find,
        map_fil_red::map_fil_red,
    },
    ParIter, Params,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{ConIterOfVec, ConcurrentIter, IntoConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;

/// An iterator that maps the elements of the iterator with a given map function.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`ParMap::num_threads`] and [`ParMap::chunk_size`] methods.
pub struct ParMapFilter<I, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
    F: Fn(&O) -> bool + Send + Sync + Clone,
{
    iter: I,
    params: Params,
    map: M,
    filter: F,
}

impl<I, O, M, F> ParIter for ParMapFilter<I, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter. Default is temporary also
    M: Fn(I::Item) -> O + Send + Sync + Clone,
    F: Fn(&O) -> bool + Send + Sync + Clone,
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

    fn map<O2, M2>(self, map: M2) -> ParMap<impl ConcurrentIter<Item = O>, O2, M2>
    where
        O2: Send + Sync + Default,
        M2: Fn(Self::Item) -> O2 + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParMap::new(iter, params, map)
    }

    fn flat_map<O2, OI, FM>(self, fmap: FM) -> ParFMap<ConIterOfVec<O>, O2, OI, FM>
    where
        O2: Send + Sync + Default,
        OI: IntoIterator<Item = O2>,
        FM: Fn(Self::Item) -> OI + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParFMap::new(iter, params, fmap)
    }

    fn filter<F2>(
        self,
        filter: F2,
    ) -> ParMapFilter<I, O, M, impl Fn(&O) -> bool + Send + Sync + Clone>
    where
        F2: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        let (params, iter, map, filter1) = (self.params, self.iter, self.map, self.filter);
        let composed = move |x: &O| filter1(x) && filter(x);
        ParMapFilter::new(iter, params, map, composed)
    }

    fn count(self) -> usize {
        let (params, iter, map, filter) = (self.params, self.iter, self.map, self.filter);
        map_fil_cnt(params, iter, map, filter)
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
        output.map_filter_into(self)
    }
}

impl<I, O, M, F> ParMapFilter<I, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
    F: Fn(&O) -> bool + Send + Sync + Clone,
{
    // define

    pub(crate) fn new(iter: I, params: Params, map: M, filter: F) -> Self {
        Self {
            iter,
            params,
            map,
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
        let (params, iter, map, filter) = (self.params, self.iter, self.map, self.filter);

        match params.is_sequential() {
            true => seq_map_fil_col(iter, map, filter, push),
            _ => {
                let bag = ConcurrentBag::new();
                match iter.try_get_len() {
                    Some(len) => {
                        let positions = ConcurrentOrderedBag::with_fixed_capacity(len);
                        let (bag, positions) =
                            par_map_fil_col(params, iter, map, filter, bag, positions);
                        merge_bag_and_positions(bag, &positions, &mut push);
                    }
                    None => {
                        let positions = ConcurrentOrderedBag::new();
                        let (bag, positions) =
                            par_map_fil_col(params, iter, map, filter, bag, positions);
                        merge_bag_and_positions(bag, &positions, &mut push);
                    }
                };
            }
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
    pub fn first_with_index(self) -> Option<(usize, O)> {
        let (params, iter, map, filter) = (self.params, self.iter, self.map, self.filter);
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
        let (params, iter, map, filter) = (self.params, self.iter, self.map, self.filter);
        let composed = move |x: &O| filter(x) && predicate(x);
        map_fil_find(params, iter, map, composed)
    }
}

impl<I, O, M, F> Reduce<O> for ParMapFilter<I, O, M, F>
where
    I: ConcurrentIter,
    O: Send + Sync + Default,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
    F: Fn(&O) -> bool + Send + Sync + Clone,
{
    fn reduce<R>(self, reduce: R) -> Option<O>
    where
        R: Fn(O, O) -> O + Send + Sync,
    {
        map_fil_red(self.params, self.iter, self.map, self.filter, reduce)
    }
}
