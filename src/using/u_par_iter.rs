use crate::{
    ChunkSize, DefaultRunner, IterationOrder, NumThreads, ParCollectInto, ParallelRunner, Params,
    Sum,
    using::{
        Using,
        computations::{u_map_clone, u_map_copy, u_map_count, u_reduce_sum, u_reduce_unit},
    },
};
use core::cmp::Ordering;
use orx_concurrent_iter::ConcurrentIter;

/// Parallel iterator which allows mutable access to a variable of type `U` within its iterator methods.
///
/// Note that one variable will be created per thread used by the parallel computation.
pub trait ParIterUsing<U, R = DefaultRunner>: Sized + Send + Sync
where
    R: ParallelRunner,
    U: Using,
{
    /// Element type of the parallel iterator.
    type Item;

    /// Returns a reference to the input concurrent iterator.
    fn con_iter(&self) -> &impl ConcurrentIter;

    /// Parameters of the parallel iterator.
    ///
    /// See [crate::ParIter::params] for details.
    fn params(&self) -> Params;

    // params transformations

    /// Sets the number of threads to be used in the parallel execution.
    /// Integers can be used as the argument with the following mapping:
    ///
    /// * `0` -> `NumThreads::Auto`
    /// * `1` -> `NumThreads::sequential()`
    /// * `n > 0` -> `NumThreads::Max(n)`
    ///
    ///     /// Parameters of the parallel iterator.
    ///
    /// See [crate::ParIter::num_threads] for details.
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets the number of elements to be pulled from the concurrent iterator during the
    /// parallel execution. When integers are used as argument, the following mapping applies:
    ///
    /// * `0` -> `ChunkSize::Auto`
    /// * `n > 0` -> `ChunkSize::Exact(n)`
    ///
    /// Please use the default enum constructor for creating `ChunkSize::Min` variant.
    ///
    /// See [crate::ParIter::chunk_size] for details.
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets the iteration order of the parallel computation.
    ///
    /// See [crate::ParIter::iteration_order] for details.
    fn iteration_order(self, collect: IterationOrder) -> Self;

    /// Rather than the [`DefaultRunner`], uses the parallel runner `Q` which implements [`ParallelRunner`].
    ///
    /// See [crate::ParIter::with_runner] for details.
    fn with_runner<Q: ParallelRunner>(self) -> impl ParIterUsing<U, Q, Item = Self::Item>;

    // computation transformations

    /// Takes a closure `map` and creates a parallel iterator which calls that closure on each element.
    ///
    /// Unlike [crate::ParIter::map], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn map<Out, Map>(self, map: Map) -> impl ParIterUsing<U, R, Item = Out>
    where
        Map: Fn(&mut U::Item, Self::Item) -> Out + Sync + Clone;

    /// Creates an iterator which uses a closure `filter` to determine if an element should be yielded.
    ///
    /// Unlike [crate::ParIter::filter], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn filter<Filter>(self, filter: Filter) -> impl ParIterUsing<U, R, Item = Self::Item>
    where
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Sync + Clone;

    /// Creates an iterator that works like map, but flattens nested structure.
    ///
    /// Unlike [crate::ParIter::flat_map], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterUsing<U, R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Sync + Clone;

    /// Creates an iterator that both filters and maps.
    ///
    /// The returned iterator yields only the values for which the supplied closure `filter_map` returns `Some(value)`.
    ///
    /// `filter_map` can be used to make chains of `filter` and `map` more concise.
    /// The example below shows how a `map().filter().map()` can be shortened to a single call to `filter_map`.
    ///
    /// Unlike [crate::ParIter::filter_map], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterUsing<U, R, Item = Out>
    where
        FilterMap: Fn(&mut U::Item, Self::Item) -> Option<Out> + Sync + Clone;

    /// Does something with each element of an iterator, passing the value on.
    ///
    /// Unlike [crate::ParIter::inspect], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn inspect<Operation>(self, operation: Operation) -> impl ParIterUsing<U, R, Item = Self::Item>
    where
        Operation: Fn(&mut U::Item, &Self::Item) + Sync + Clone,
    {
        let map = move |u: &mut U::Item, x: Self::Item| {
            operation(u, &x);
            x
        };
        self.map(map)
    }

    // special item transformations

    /// Creates an iterator which copies all of its elements.
    ///
    /// Unlike [crate::ParIter::copied], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn copied<'a, T>(self) -> impl ParIterUsing<U, R, Item = T>
    where
        T: 'a + Copy,
        Self: ParIterUsing<U, R, Item = &'a T>,
    {
        self.map(u_map_copy)
    }

    /// Creates an iterator which clones all of its elements.
    ///
    /// Unlike [crate::ParIter::cloned], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn cloned<'a, T>(self) -> impl ParIterUsing<U, R, Item = T>
    where
        T: 'a + Clone,
        Self: ParIterUsing<U, R, Item = &'a T>,
    {
        self.map(u_map_clone)
    }

    /// Creates an iterator that flattens nested structure.
    ///
    /// Unlike [crate::ParIter::flatten], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn flatten(self) -> impl ParIterUsing<U, R, Item = <Self::Item as IntoIterator>::Item>
    where
        Self::Item: IntoIterator,
    {
        let map = |_: &mut U::Item, e: Self::Item| e.into_iter();
        self.flat_map(map)
    }

    // collect

    /// Collects all the items from an iterator into a collection.
    ///
    /// Unlike [crate::ParIter::collect_into], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>;

    /// Transforms an iterator into a collection.
    ///
    /// Unlike [crate::ParIter::collect], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let output = C::empty(self.con_iter().try_get_len());
        self.collect_into(output)
    }

    // reduce

    /// Reduces the elements to a single one, by repeatedly applying a reducing operation.
    ///
    /// See the details here: [crate::ParIter::reduce].
    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync;

    /// Tests if every element of the iterator matches a predicate.
    ///
    /// Unlike [crate::ParIter::all], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn all<Predicate>(self, predicate: Predicate) -> bool
    where
        Self::Item: Send,
        Predicate: Fn(&mut U::Item, &Self::Item) -> bool + Sync + Clone,
    {
        let violates = |u: &mut U::Item, x: &Self::Item| !predicate(u, x);
        self.find(violates).is_none()
    }

    /// Tests if any element of the iterator matches a predicate.
    ///
    /// Unlike [crate::ParIter::any], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn any<Predicate>(self, predicate: Predicate) -> bool
    where
        Self::Item: Send,
        Predicate: Fn(&mut U::Item, &Self::Item) -> bool + Sync + Clone,
    {
        self.find(predicate).is_some()
    }

    /// Consumes the iterator, counting the number of iterations and returning it.
    ///
    /// See the details here: [crate::ParIter::count].
    fn count(self) -> usize {
        self.map(u_map_count).reduce(u_reduce_sum).unwrap_or(0)
    }

    /// Calls a closure on each element of an iterator.
    ///
    /// Unlike [crate::ParIter::for_each], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn for_each<Operation>(self, operation: Operation)
    where
        Operation: Fn(&mut U::Item, Self::Item) + Sync,
    {
        let map = |u: &mut U::Item, x| operation(u, x);
        let _ = self.map(map).reduce(u_reduce_unit);
    }

    /// Returns the maximum element of an iterator.
    ///
    /// See the details here: [crate::ParIter::max].
    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(|_, a, b| Ord::max(a, b))
    }

    /// Returns the element that gives the maximum value with respect to the specified `compare` function.
    ///
    /// See the details here: [crate::ParIter::max_by].
    fn max_by<Compare>(self, compare: Compare) -> Option<Self::Item>
    where
        Self::Item: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |_: &mut U::Item, x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the maximum value from the specified function.
    ///
    /// See the details here: [crate::ParIter::max_by_key].
    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Option<Self::Item>
    where
        Self::Item: Send,
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |_: &mut U::Item, x, y| match key(&x).cmp(&key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns the minimum element of an iterator.
    ///
    /// See the details here: [crate::ParIter::min].
    fn min(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(|_, a, b| Ord::min(a, b))
    }

    /// Returns the element that gives the minimum value with respect to the specified `compare` function.
    ///
    /// See the details here: [crate::ParIter::min_by].
    fn min_by<Compare>(self, compare: Compare) -> Option<Self::Item>
    where
        Self::Item: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |_: &mut U::Item, x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the minimum value from the specified function.
    ///
    /// See the details here: [crate::ParIter::min_by_key].
    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Option<Self::Item>
    where
        Self::Item: Send,
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |_: &mut U::Item, x, y| match get_key(&x).cmp(&get_key(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Sums the elements of an iterator.
    ///
    /// See the details here: [crate::ParIter::sum].
    fn sum<Out>(self) -> Out
    where
        Self::Item: Sum<Out>,
        Out: Send,
    {
        self.map(Self::Item::u_map)
            .reduce(Self::Item::u_reduce)
            .unwrap_or(Self::Item::zero())
    }

    // early exit

    /// Returns the first (or any) element of the iterator; returns None if it is empty.
    ///
    /// * first element is returned if default iteration order `IterationOrder::Ordered` is used,
    /// * any element is returned if `IterationOrder::Arbitrary` is set.
    ///
    /// See the details here: [crate::ParIter::first].
    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send;

    /// Searches for an element of an iterator that satisfies a `predicate`.
    ///
    /// Unlike [crate::ParIter::find], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn find<Predicate>(self, predicate: Predicate) -> Option<Self::Item>
    where
        Self::Item: Send,
        Predicate: Fn(&mut U::Item, &Self::Item) -> bool + Sync,
    {
        self.filter(&predicate).first()
    }
}
