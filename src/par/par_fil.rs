use super::{
    collect_into::par_map_fil_collect_into::ParMapFilterCollectInto, par_fmap::ParFMap,
    par_map::ParMap, reduce::Reduce,
};
use crate::{
    core::{
        default_fns::map_self, map_fil_cnt::map_fil_cnt, map_fil_find::map_fil_find,
        map_fil_red::map_fil_red,
    },
    ParIter, ParMapFilter, Params,
};
use orx_concurrent_iter::{ConIterOfVec, ConcurrentIter, IntoConcurrentIter};
use orx_split_vec::SplitVec;

/// An iterator that maps the elements of the iterator with a given map function.
///
/// The iterator can be executed in parallel or sequentially with different chunk sizes; see [`ParMap::num_threads`] and [`ParMap::chunk_size`] methods.
pub struct ParFilter<I, F>
where
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync + Clone,
{
    iter: I,
    params: Params,
    filter: F,
}

impl<I, F> ParIter for ParFilter<I, F>
where
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync + Clone,
    I::Item: Default,
{
    type Item = I::Item;

    fn num_threads(mut self, num_threads: impl Into<crate::NumThreads>) -> Self {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<crate::ChunkSize>) -> Self {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    fn map<O, M>(self, map: M) -> ParMap<ConIterOfVec<<I as ConcurrentIter>::Item>, O, M>
    where
        O: Send + Sync + Default,
        M: Fn(Self::Item) -> O + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParMap::new(iter, params, map)
    }

    fn flat_map<O, OI, FM>(
        self,
        fmap: FM,
    ) -> ParFMap<ConIterOfVec<<I as ConcurrentIter>::Item>, O, OI, FM>
    where
        O: Send + Sync + Default,
        OI: IntoIterator<Item = O>,
        FM: Fn(Self::Item) -> OI + Send + Sync + Clone,
    {
        let params = self.params;
        let vec = self.collect_vec();
        let iter = vec.into_con_iter();
        ParFMap::new(iter, params, fmap)
    }

    fn filter<F2>(self, filter: F2) -> ParFilter<I, impl Fn(&I::Item) -> bool + Send + Sync + Clone>
    where
        F2: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        let (params, iter, filter1) = (self.params, self.iter, self.filter);
        let composed = move |x: &I::Item| filter1(x) && filter(x);
        ParFilter::new(iter, params, composed)
    }

    fn count(self) -> usize {
        let (params, iter, filter) = (self.params, self.iter, self.filter);
        map_fil_cnt(params, iter, map_self, filter)
    }
}

impl<I, F> ParFilter<I, F>
where
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync + Clone,
{
    // define

    pub(crate) fn new(iter: I, params: Params, filter: F) -> Self {
        Self {
            iter,
            params,
            filter,
        }
    }

    /// Parameters of the parallel computation which can be set by `num_threads` and `chunk_size` methods.
    pub fn params(&self) -> Params {
        self.params
    }

    // transform

    // /// Takes the closure `map` and creates an iterator which calls that closure on each element.
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// use orx_parallel::*;
    // ///
    // /// let doubles = (0..5).into_par().map(|x| x * 2).collect_vec();
    // /// assert_eq!(&doubles[..], &[0, 2, 4, 6, 8]);
    // /// ```
    // pub fn map<O, M>(self, map: M) -> ParMap<ConIterOfVec<<I as ConcurrentIter>::Item>, O, M>
    // where
    //     O: Send + Sync + Default,
    //     M: Fn(I::Item) -> O + Send + Sync + Clone,
    //     I::Item: Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter. Default is temporary also
    // {
    //     let params = self.params;
    //     let vec = self.collect_vec();
    //     let iter = vec.into_con_iter();
    //     ParMap::new(iter, params, map)
    // }

    // /// Takes the closure `fmap` and creates an iterator which calls that closure on each element and flattens the result.
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// use orx_parallel::*;
    // ///
    // /// let numbers = (0..5).into_par().flat_map(|x| vec![x; x]).collect_vec();
    // /// assert_eq!(&numbers[..], &[1, 2, 2, 3, 3, 3, 4, 4, 4, 4]);
    // /// ```
    // pub fn flat_map<O, OI, M>(
    //     self,
    //     fmap: M,
    // ) -> ParFMap<ConIterOfVec<<I as ConcurrentIter>::Item>, O, OI, M>
    // where
    //     O: Send + Sync + Default,
    //     OI: IntoIterator<Item = O>,
    //     M: Fn(I::Item) -> OI + Send + Sync + Clone,
    //     I::Item: Default,
    // {
    //     let params = self.params;
    //     let vec = self.collect_vec();
    //     let iter = vec.into_con_iter();
    //     ParFMap::new(iter, params, fmap)
    // }

    // /// Creates an iterator which uses the closure `filter` to determine if an element should be yielded.
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// use orx_parallel::*;
    // ///
    // /// let evens = (0..10).into_par().filter(|x| x % 2 == 0).collect_vec();
    // /// assert_eq!(&evens[..], &[0, 2, 4, 6, 8]);
    // /// ```
    // pub fn filter<F2>(
    //     self,
    //     filter: F2,
    // ) -> ParFilter<I, impl Fn(&I::Item) -> bool + Send + Sync + Clone>
    // where
    //     F2: Fn(&I::Item) -> bool + Send + Sync + Clone,
    // {
    //     let (params, iter, filter1) = (self.params, self.iter, self.filter);
    //     let composed = move |x: &I::Item| filter1(x) && filter(x);
    //     ParFilter::new(iter, params, composed)
    // }

    // collect

    /// Transforms the iterator into a collection.
    ///
    /// In this case, the result is transformed into a standard vector; i.e., `std::vec::Vec`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let evens = (0..10).into_par().filter(|x| x % 2 == 0).collect_vec();
    /// assert_eq!(evens, vec![0, 2, 4, 6, 8]);
    /// ```
    pub fn collect_vec(self) -> Vec<I::Item>
    where
        I::Item: Default,
    {
        ParMapFilter::new(self.iter, self.params, map_self, self.filter).collect_vec()
    }

    /// Transforms the iterator into a collection.
    ///
    /// In this case, the result is transformed into the split vector which is the underlying [`PinnedVec`](https://crates.io/crates/orx-pinned-vec) used to collect the results concurrently;
    /// i.e., [`SplitVec`](https://crates.io/crates/orx-split-vec).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use orx_split_vec::*;
    ///
    /// let evens = (0..10).into_par().filter(|x| x % 2 == 0).collect();
    /// assert_eq!(evens, SplitVec::from_iter([0, 2, 4, 6, 8]));
    /// ```
    pub fn collect(self) -> SplitVec<I::Item>
    where
        I::Item: Default,
    {
        ParMapFilter::new(self.iter, self.params, map_self, self.filter).collect()
    }

    /// Collects elements yielded by the iterator into the given `output` collection.
    ///
    /// Note that `output` does not need to be empty; hence, this method allows extending collections from the parallel iterator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use orx_split_vec::*;
    ///
    /// let output_vec = vec![42];
    ///
    /// let evens = (0..10).into_par().filter(|x| x % 2 == 0);
    /// let output_vec = evens.collect_into(output_vec);
    /// assert_eq!(output_vec, vec![42, 0, 2, 4, 6, 8]);
    ///
    /// let odds = (0..10).into_par().filter(|x| x % 2 == 1);
    /// let output_vec = odds.collect_into(output_vec);
    /// assert_eq!(output_vec, vec![42, 0, 2, 4, 6, 8, 1, 3, 5, 7, 9]);
    ///
    /// // alternatively, any `PinnedVec` can be used
    /// let output_vec: SplitVec<_> = [42].into_iter().collect();
    ///
    /// let evens = (0..10).into_par().filter(|x| x % 2 == 0);
    /// let output_vec = evens.collect_into(output_vec);
    /// assert_eq!(output_vec, vec![42, 0, 2, 4, 6, 8]);
    /// ```
    pub fn collect_into<C: ParMapFilterCollectInto<I::Item>>(self, output: C) -> C
    where
        I::Item: Default,
    {
        ParMapFilter::new(self.iter, self.params, map_self, self.filter).collect_into(output)
    }

    // count

    /// Consumes the iterator, counting the number of iterations and returning it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let evens = (0..10).into_par().filter(|x| x % 2 == 0);
    /// assert_eq!(evens.count(), 5);
    /// ```
    pub fn count(self) -> usize {
        let (params, iter, filter) = (self.params, self.iter, self.filter);
        map_fil_cnt(params, iter, map_self, filter)
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

    /// Returns the first element of the iterator; returns None if the iterator is empty.
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
    /// let first_prime = (21..100).into_par().filter(is_prime).first();
    /// assert_eq!(first_prime, Some(23));
    ///
    /// let first_prime = (24..28).into_par().filter(is_prime).first();
    /// assert_eq!(first_prime, None);
    /// ```
    pub fn first(self) -> Option<I::Item> {
        self.first_with_index().map(|x| x.1)
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

    /// Returns the first element of the iterator satisfying the given `predicate`; returns None if the iterator is empty.
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
    /// let first_prime = (21..100).into_par().find(is_prime);
    /// assert_eq!(first_prime, Some(23));
    ///
    /// let first_prime = (24..28).into_par().find(is_prime);
    /// assert_eq!(first_prime, None);
    /// ```
    pub fn find<P>(self, predicate: P) -> Option<I::Item>
    where
        P: Fn(&I::Item) -> bool + Send + Sync,
    {
        self.find_with_index(predicate).map(|x| x.1)
    }
}

impl<I, F> Reduce<I::Item> for ParFilter<I, F>
where
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync + Clone,
{
    fn reduce<R>(self, reduce: R) -> Option<I::Item>
    where
        R: Fn(I::Item, I::Item) -> I::Item + Send + Sync,
    {
        map_fil_red(self.params, self.iter, map_self, self.filter, reduce)
    }
}
