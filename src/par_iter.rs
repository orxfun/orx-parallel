use crate::{
    Params,
    collect_into::ParCollectInto,
    computations::{map_clone, map_copy, map_count, reduce_sum, reduce_unit},
    parameters::{ChunkSize, CollectOrdering, NumThreads},
    runner::{DefaultRunner, ParallelRunner},
    special_type_sets::Sum,
};
use orx_concurrent_iter::ConcurrentIter;
use std::cmp::Ordering;

/// Parallel iterator.
pub trait ParIter<R = DefaultRunner>: Sized + Send + Sync
where
    R: ParallelRunner,
{
    /// Element type of the parallel iterator.
    type Item: Send + Sync;

    /// Returns a reference to the input concurrent iterator.
    fn con_iter(&self) -> &impl ConcurrentIter;

    /// Parameters of the parallel iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// assert_eq!(
    ///     vec.par().params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(1).params(),
    ///     &Params::new(1, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(1).chunk_size(64).params(),
    ///     &Params::new(1, 64, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par()
    ///         .num_threads(1)
    ///         .chunk_size(64)
    ///         .collect_ordering(CollectOrdering::Arbitrary)
    ///         .params(),
    ///     &Params::new(1, 64, CollectOrdering::Arbitrary)
    /// );
    /// ```
    fn params(&self) -> &Params;

    // params transformations

    /// Sets the number of threads to be used in the parallel execution.
    /// Integers can be used as the argument with the following mapping:
    ///
    /// * `0` -> `NumThreads::Auto`
    /// * `1` -> `NumThreads::sequential()`
    /// * `n > 0` -> `NumThreads::Max(n)`
    ///
    /// See [`NumThreads`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// // all available threads can be used
    /// assert_eq!(
    ///     vec.par().params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(0).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// // maximum 4 threads can be used
    /// assert_eq!(
    ///     vec.par().num_threads(4).chunk_size(64).params(),
    ///     &Params::new(4, 64, CollectOrdering::Ordered)
    /// );
    /// ```
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets the number of elements to be pulled from the concurrent iterator during the
    /// parallel execution. When integers are used as argument, the following mapping applies:
    ///
    /// * `0` -> `ChunkSize::Auto`
    /// * `n > 0` -> `NumThreads::Min(n)`
    ///
    /// See [`ChunkSize`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    /// use std::num::NonZero;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// // chunk sizes will be dynamically decided by the parallel runner
    /// assert_eq!(
    ///     vec.par().params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().chunk_size(0).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// // minimum chunk size will be 64, but can be dynamically increased by the parallel runner
    /// assert_eq!(
    ///     vec.par().chunk_size(64).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Min(NonZero::new(64).unwrap()), CollectOrdering::Ordered)
    /// );
    ///
    /// // chunk size will always be 64, parallel runner cannot change
    /// assert_eq!(
    ///     vec.par().chunk_size(ChunkSize::Exact(NonZero::new(64).unwrap())).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Exact(NonZero::new(64).unwrap()), CollectOrdering::Ordered)
    /// );
    /// ```
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets the collect ordering of the parallel computation.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// /// results are collected in order consistent to the input order
    /// assert_eq!(
    ///     vec.par().params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().collect_ordering(CollectOrdering::Ordered).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// /// results might be collected in arbitrary order
    /// assert_eq!(
    ///     vec.par().collect_ordering(CollectOrdering::Arbitrary).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Arbitrary)
    /// );
    /// ```
    fn collect_ordering(self, collect: CollectOrdering) -> Self;

    /// Rather than the [`DefaultRunner`], uses the parallel runner `Q` which implements [`ParallelRunner`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use orx_parallel::*;
    ///
    /// let inputs = vec![1, 2, 3, 4];
    ///
    /// // uses the default runner
    /// let sum = inputs.par().sum();
    ///
    /// // uses the custom parallel runner MyParallelRunner: ParallelRunner
    /// let sum = inputs.par().with_runner::<MyParallelRunner>().sum();
    /// ```
    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item>;

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone;

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Send + Sync + Clone;

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(Self::Item) -> IOut + Send + Sync + Clone;

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(Self::Item) -> Option<Out> + Send + Sync + Clone;

    fn inspect<Operation>(self, operation: Operation) -> impl ParIter<R, Item = Self::Item>
    where
        Operation: Fn(&Self::Item) + Sync + Send + Clone,
    {
        let map = move |x| {
            operation(&x);
            x
        };
        self.map(map)
    }

    // special item transformations

    fn copied<'a, T>(self) -> impl ParIter<R, Item = T>
    where
        T: 'a + Copy + Send + Sync,
        Self: ParIter<R, Item = &'a T>,
    {
        self.map(map_copy)
    }

    fn cloned<'a, T>(self) -> impl ParIter<R, Item = T>
    where
        T: 'a + Clone + Send + Sync,
        Self: ParIter<R, Item = &'a T>,
    {
        self.map(map_clone)
    }

    fn flatten(self) -> impl ParIter<R, Item = <Self::Item as IntoIterator>::Item>
    where
        Self::Item: IntoIterator,
        <Self::Item as IntoIterator>::IntoIter: Send + Sync,
        <Self::Item as IntoIterator>::Item: Send + Sync,
        R: Send + Sync,
        Self: Send + Sync,
    {
        let map = |e: Self::Item| e.into_iter();
        self.flat_map(map)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>;

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let output = C::empty(self.con_iter().try_get_len());
        self.collect_into(output)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync;

    fn all<Predicate>(self, predicate: Predicate) -> bool
    where
        Predicate: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        let violates = |x: &Self::Item| !predicate(x);
        self.find(violates).is_none()
    }

    fn any<Predicate>(self, predicate: Predicate) -> bool
    where
        Predicate: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        self.find(predicate).is_some()
    }

    fn count(self) -> usize {
        self.fold(map_count, reduce_sum).unwrap_or(0)
    }

    fn fold<Map, Reduce, Out>(self, map: Map, reduce: Reduce) -> Option<Out>
    where
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
        Reduce: Fn(Out, Out) -> Out + Send + Sync,
        Out: Send + Sync,
    {
        self.map(map).reduce(reduce)
    }

    fn for_each<Operation>(self, operation: Operation)
    where
        Operation: Fn(Self::Item) + Sync + Send,
    {
        let map = |x| operation(x);
        let _ = self.fold(map, reduce_unit);
    }

    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.reduce(Ord::max)
    }

    fn max_by<Compare>(self, compare: Compare) -> Option<Self::Item>
    where
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Option<Self::Item>
    where
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |x, y| match key(&x).cmp(&key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn min_by<Compare>(self, compare: Compare) -> Option<Self::Item>
    where
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn min(self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.reduce(Ord::min)
    }

    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Option<Self::Item>
    where
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |x, y| match get_key(&x).cmp(&get_key(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn sum<Out>(self) -> Out
    where
        Self::Item: Sum<Out>,
        Out: Send + Sync,
    {
        self.fold(Self::Item::map, Self::Item::reduce)
            .unwrap_or(Self::Item::zero())
    }

    // early exit

    fn next(self) -> Option<Self::Item>;

    fn any_element(self) -> Option<Self::Item>;

    fn find<Predicate>(self, predicate: Predicate) -> Option<Self::Item>
    where
        Predicate: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        self.filter(predicate).next()
    }

    fn find_any<Predicate>(self, predicate: Predicate) -> Option<Self::Item>
    where
        Predicate: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        self.filter(predicate).any_element()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn abc() {
        let vec = vec![1, 2, 3, 4];

        assert_eq!(
            vec.par().params(),
            &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
        );

        assert_eq!(
            vec.par().num_threads(1).params(),
            &Params::new(1, ChunkSize::Auto, CollectOrdering::Ordered)
        );

        assert_eq!(
            vec.par().num_threads(1).chunk_size(64).params(),
            &Params::new(1, 64, CollectOrdering::Ordered)
        );

        assert_eq!(
            vec.par()
                .num_threads(1)
                .chunk_size(64)
                .collect_ordering(CollectOrdering::Arbitrary)
                .params(),
            &Params::new(1, 64, CollectOrdering::Arbitrary)
        );
    }
}
