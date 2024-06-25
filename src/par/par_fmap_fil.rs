use super::{
    collect_into::par_fmap_fil_collect_into::{merge_bag_and_pos_len, ParFMapFilterCollectInto},
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
    // pub fn map<O2, M2>(self, map: M2) -> ParMap<ConIterOfVec<O>, O2, M2>
    // where
    //     O2: Send + Sync + Default,
    //     M2: Fn(O) -> O2 + Send + Sync + Clone,
    //     O: Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter. Default is temporary also
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
    // pub fn flat_map<OI2, M2>(self, fmap: M2) -> ParFMap<ConIterOfVec<O>, OI2::Item, OI2, M2>
    // where
    //     M2: Fn(O) -> OI2 + Send + Sync + Clone,
    //     OI2: IntoIterator<Item = O>,
    //     O: Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter. Default is temporary also
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
    // ) -> ParFMapFilter<I, O, OI, M, impl Fn(&O) -> bool + Send + Sync>
    // where
    //     F2: Fn(&O) -> bool + Send + Sync,
    // {
    //     let (params, iter, fmap, filter1) = (self.params, self.iter, self.fmap, self.filter);
    //     let composed = move |x: &O| filter1(x) && filter(x);
    //     ParFMapFilter::new(iter, params, fmap, composed)
    // }

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
    pub fn collect_vec(self) -> Vec<O>
    where
        O: Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter
    {
        let mut vec = vec![];
        self.collect_bag(|x| vec.push(x));
        vec
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
    pub fn collect(self) -> SplitVec<O>
    where
        O: Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter
    {
        let mut vec = SplitVec::new();
        self.collect_bag(|x| vec.push(x));
        vec
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
    pub fn collect_into<C: ParFMapFilterCollectInto<O>>(self, output: C) -> C
    where
        O: Default,
    {
        output.fmap_filter_into(self)
    }

    // collect-x

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

    /// Transforms the iterator into a collection.
    ///
    /// In this case, the result is transformed into a standard vector; i.e., `std::vec::Vec`.
    ///
    /// `collect_x_vec` differs from `collect_vec` method by the following:
    /// * `collect_vec` will  return a result which contains yielded elements in the same order. Therefore, it results in a deterministic output.
    /// `collect_x_vec`, on the other hand, does not try to preserve the order. The order of elements in the output depends on the execution speeds of different threads.
    /// * `collect_x_vec` might perform faster than `collect_vec` in certain situations.
    ///
    /// Due to above `collect_x_vec` can be preferred over `collect_vec` in performance-critical operations where the order of elements in the output is not important.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let mut output = (0..5).into_par().flat_map(|x| vec![x; x]).collect_x_vec();
    /// output.sort();
    /// assert_eq!(output, vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4]);
    /// ```
    pub fn collect_x_vec(self) -> Vec<O>
    where
        O: Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter
    {
        self.collect_bag_x(ConcurrentBag::new()).into_inner().into()
    }

    /// Transforms the iterator into a collection.
    ///
    /// In this case, the result is transformed into the split vector which is the underlying [`PinnedVec`](https://crates.io/crates/orx-pinned-vec) used to collect the results concurrently;
    /// i.e., [`SplitVec`](https://crates.io/crates/orx-split-vec).
    ///
    /// `collect_x` differs from `collect` method by the following:
    /// * `collect` will  return a result which contains yielded elements in the same order. Therefore, it results in a deterministic output.
    /// `collect_x`, on the other hand, does not try to preserve the order. The order of elements in the output depends on the execution speeds of different threads.
    /// * `collect_x` might perform faster than `collect` in certain situations.
    ///
    /// Due to above `collect_x` can be preferred over `collect` in performance-critical operations where the order of elements in the output is not important.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use orx_split_vec::*;
    ///
    /// let output = (0..5).into_par().flat_map(|x| vec![x; x]).collect_x();
    /// let mut sorted_output = output.to_vec();
    /// sorted_output.sort(); // WIP: PinnedVec::sort(&mut self)
    /// assert_eq!(sorted_output, vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4]);
    /// ```
    pub fn collect_x(self) -> SplitVec<O>
    where
        O: Default, // todo!: temporary requirement, must replace with PinnedVec::into_iter
    {
        self.collect_bag_x(ConcurrentBag::new()).into_inner()
    }

    /// Collects elements yielded by the iterator into the given `output` collection.
    ///
    /// Note that `output` does not need to be empty; hence, this method allows extending collections from the parallel iterator.
    ///
    /// `collect_x_into` differs from `collect_into` method by the following:
    /// * `collect_into` will  return a result which contains yielded elements in the same order. Therefore, it results in a deterministic output.
    /// `collect_x_into`, on the other hand, does not try to preserve the order. The order of elements in the output depends on the execution speeds of different threads.
    /// * `collect_x_into` might perform faster than `collect_into` in certain situations.
    ///
    /// Due to above `collect_x_into` can be preferred over `collect_into` in performance-critical operations where the order of elements in the output is not important.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use orx_split_vec::*;
    ///
    /// let mut output = SplitVec::with_doubling_growth_and_fragments_capacity(32);
    /// output.push(42);
    ///
    /// let output = (0..5).into_par().flat_map(|x| vec![x; x]).collect_x_into(output);
    /// let mut sorted_output = output.to_vec();
    /// sorted_output.sort(); // WIP: PinnedVec::sort(&mut self)
    /// assert_eq!(sorted_output, vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4, 42]);
    /// ```
    pub fn collect_x_into<P, B: Into<ConcurrentBag<O, P>>>(self, output: B) -> P
    where
        O: Default,
        P: PinnedVec<O>,
    {
        self.collect_bag_x(output.into()).into_inner()
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
        let (params, iter, fmap, filter) = (self.params, self.iter, self.fmap, self.filter);
        fmap_fil_cnt(params, iter, fmap, filter)
    }

    // find

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
    pub fn first(self) -> Option<O> {
        let (params, iter, fmap, filter) = (self.params, self.iter, self.fmap, self.filter);
        fmap_fil_find(params, iter, fmap, filter)
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
    pub fn find<P>(self, predicate: P) -> Option<O>
    where
        P: Fn(&O) -> bool + Send + Sync,
    {
        let (params, iter, fmap, filter) = (self.params, self.iter, self.fmap, self.filter);
        let composed = move |x: &O| filter(x) && predicate(x);
        fmap_fil_find(params, iter, fmap, composed)
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
