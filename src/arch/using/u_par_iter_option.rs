use crate::default_fns::{u_map_count, u_reduce_sum, u_reduce_unit};
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::using::Using;
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParThreadPool, RunnerWithPool, Sum,
};
use core::cmp::Ordering;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with None.
///
/// Unlike [crate::ParIterOption], the threads have access to a mutable reference of the used variable in each thread.
///
/// Please see [`crate::ParIterUsing`] for details and examples.
///
/// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
pub trait ParIterOptionUsing<'using, U, R = DefaultRunner>
where
    R: ParallelRunner,
    U: Using<'using>,
{
    /// Type of the success element, to be received as the Some variant iff the entire computation succeeds.
    type Item;

    // params transformations

    /// Sets the number of threads to be used in the parallel execution.
    /// Integers can be used as the argument with the following mapping:
    ///
    /// * `0` -> `NumThreads::Auto`
    /// * `1` -> `NumThreads::sequential()`
    /// * `n > 0` -> `NumThreads::Max(n)`
    ///
    /// See [`NumThreads`] and [`crate::ParIter::num_threads`] for details.
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets the number of elements to be pulled from the concurrent iterator during the
    /// parallel execution. When integers are used as argument, the following mapping applies:
    ///
    /// * `0` -> `ChunkSize::Auto`
    /// * `n > 0` -> `ChunkSize::Exact(n)`
    ///
    /// Please use the default enum constructor for creating `ChunkSize::Min` variant.
    ///
    /// See [`ChunkSize`] and [`crate::ParIter::chunk_size`] for details.
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets the iteration order of the parallel computation.
    ///
    /// See [`IterationOrder`] and [`crate::ParIter::iteration_order`] for details.
    fn iteration_order(self, order: IterationOrder) -> Self;

    /// Rather than the [`DefaultRunner`], uses the parallel runner `Q` which implements [`ParallelRunner`].
    ///
    /// See [`ParIter::with_runner`] for details.
    ///
    /// [`DefaultRunner`]: crate::DefaultRunner
    /// [`ParIter::with_runner`]: crate::ParIter::with_runner
    fn with_runner<Q: ParallelRunner>(
        self,
        orchestrator: Q,
    ) -> impl ParIterOptionUsing<'using, U, Q, Item = Self::Item>;

    /// Rather than [`DefaultPool`], uses the parallel runner with the given `pool` implementing
    /// [`ParThreadPool`].
    ///
    /// See [`ParIter::with_pool`] for details.
    ///
    /// [`DefaultPool`]: crate::DefaultPool
    /// [`ParIter::with_pool`]: crate::ParIter::with_pool
    fn with_pool<P: ParThreadPool>(
        self,
        pool: P,
    ) -> impl ParIterOptionUsing<'using, U, RunnerWithPool<P, R::Executor>, Item = Self::Item>
    where
        Self: Sized,
    {
        let runner = RunnerWithPool::from(pool).with_executor::<R::Executor>();
        self.with_runner(runner)
    }

    // computation transformations

    /// Takes a closure `map` and creates a parallel iterator which calls that closure on each element.
    ///
    /// Transformation is only for the success path where all elements are of the `Some` variant.
    /// Any observation of a `None` case short-circuits the computation and immediately returns None.
    ///
    /// Unlike [crate::ParIterOption::map], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn map<Out, Map>(self, map: Map) -> impl ParIterOptionUsing<'using, U, R, Item = Out>
    where
        Self: Sized,
        Map: Fn(&mut U::Item, Self::Item) -> Out + Sync + Clone,
        Out: Send;

    /// Creates an iterator which uses a closure `filter` to determine if an element should be yielded.
    ///
    /// Transformation is only for the success path where all elements are of the `Some` variant.
    /// Any observation of a `None` case short-circuits the computation and immediately returns None.
    ///
    /// Unlike [crate::ParIterOption::filter], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn filter<Filter>(
        self,
        filter: Filter,
    ) -> impl ParIterOptionUsing<'using, U, R, Item = Self::Item>
    where
        Self: Sized,
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Sync + Clone,
        Self::Item: Send;

    /// Creates an iterator that works like map, but flattens nested structure.
    ///
    /// Transformation is only for the success path where all elements are of the `Some` variant.
    /// Any observation of a `None` case short-circuits the computation and immediately returns None.
    ///
    /// Unlike [crate::ParIterOption::flat_map], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterOptionUsing<'using, U, R, Item = IOut::Item>
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Sync + Clone;

    /// Creates an iterator that both filters and maps.
    ///
    /// The returned iterator yields only the values for which the supplied closure `filter_map` returns `Some(value)`.
    ///
    /// `filter_map` can be used to make chains of `filter` and `map` more concise.
    ///
    /// Unlike [crate::ParIterOption::filter_map], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterOptionUsing<'using, U, R, Item = Out>
    where
        Self: Sized,
        FilterMap: Fn(&mut U::Item, Self::Item) -> Option<Out> + Sync + Clone,
        Out: Send;

    /// Does something with each successful element of an iterator, passing the value on, provided that all elements are of Some variant;
    /// short-circuits and returns None otherwise.
    ///
    /// Unlike [crate::ParIterOption::inspect], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn inspect<Operation>(
        self,
        operation: Operation,
    ) -> impl ParIterOptionUsing<'using, U, R, Item = Self::Item>
    where
        Self: Sized,
        Operation: Fn(&mut U::Item, &Self::Item) + Sync + Clone,
        Self::Item: Send;

    // collect

    /// Collects all the items from an iterator into a collection iff all elements are of Some variant.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// Unlike [crate::ParIterOption::collect_into], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn collect_into<C>(self, output: C) -> Option<C>
    where
        Self::Item: Send,
        C: ParCollectInto<Self::Item>;

    /// Transforms an iterator into a collection iff all elements are of Ok variant.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// Similar to [`Iterator::collect`], the type annotation on the left-hand-side determines
    /// the type of the result collection; or turbofish annotation can be used.
    ///
    /// All collections implementing [`ParCollectInto`] can be used to collect into.
    ///
    /// [`ParCollectInto`]: crate::ParCollectInto
    ///
    /// Unlike [crate::ParIterOption::collect], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn collect<C>(self) -> Option<C>
    where
        Self::Item: Send,
        C: ParCollectInto<Self::Item>;

    // reduce

    /// Reduces the elements to a single one, by repeatedly applying a reducing operation.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// If the iterator is empty, returns `Some(None)`; otherwise, returns `Some` of result of the reduction.
    ///
    /// Unlike [crate::ParIterOption::reduce], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync;

    /// Tests if every element of the iterator matches a predicate.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// Unlike [crate::ParIterOption::all], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn all<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        Self: Sized,
        Self::Item: Send,
        Predicate: Fn(&mut U::Item, &Self::Item) -> bool + Sync,
    {
        let violates = |u: &mut U::Item, x: &Self::Item| !predicate(u, x);
        self.find(violates).map(|x| x.is_none())
    }

    /// Tests if any element of the iterator matches a predicate.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// Unlike [crate::ParIterOption::any], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn any<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        Self: Sized,
        Self::Item: Send,
        Predicate: Fn(&mut U::Item, &Self::Item) -> bool + Sync,
    {
        self.find(predicate).map(|x| x.is_some())
    }

    /// Consumes the iterator, counting the number of iterations and returning it.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// See the details here: [crate::ParIterOption::count].
    fn count(self) -> Option<usize>
    where
        Self: Sized,
    {
        self.map(u_map_count)
            .reduce(u_reduce_sum)
            .map(|x| x.unwrap_or(0))
    }

    /// Calls a closure on each element of an iterator, and returns `Ok(())` if all elements succeed.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// Unlike [crate::ParIterOption::for_each], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn for_each<Operation>(self, operation: Operation) -> Option<()>
    where
        Self: Sized,
        Operation: Fn(&mut U::Item, Self::Item) + Sync,
    {
        let map = |u: &mut U::Item, x: Self::Item| operation(u, x);
        self.map(map).reduce(u_reduce_unit).map(|_| ())
    }

    /// Returns Some of maximum element of an iterator if all elements succeed.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// See the details here: [crate::ParIterOption::max].
    fn max(self) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Ord + Send,
    {
        self.reduce(|_, a, b| Ord::max(a, b))
    }

    /// Returns the element that gives the maximum value with respect to the specified `compare` function.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// See the details here: [crate::ParIterOption::max_by].
    fn max_by<Compare>(self, compare: Compare) -> Option<Option<Self::Item>>
    where
        Self: Sized,
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
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// See the details here: [crate::ParIterOption::max_by_key].
    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Option<Option<Self::Item>>
    where
        Self: Sized,
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

    /// Returns Some of minimum element of an iterator if all elements succeed.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// See the details here: [crate::ParIterOption::min].
    fn min(self) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Ord + Send,
    {
        self.reduce(|_, a, b| Ord::min(a, b))
    }

    /// Returns the element that gives the minimum value with respect to the specified `compare` function.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// See the details here: [crate::ParIterOption::min_by].
    fn min_by<Compare>(self, compare: Compare) -> Option<Option<Self::Item>>
    where
        Self: Sized,
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
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// See the details here: [crate::ParIterOption::min_by_key].
    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Option<Option<Self::Item>>
    where
        Self: Sized,
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
    /// Early exits and returns None if any of the elements is None.
    ///
    /// If the iterator is empty, returns zero; otherwise, returns `Some` of the sum.
    ///
    /// See the details here: [crate::ParIterOption::sum].
    fn sum<Out>(self) -> Option<Out>
    where
        Self: Sized,
        Self::Item: Sum<Out>,
        Out: Send,
    {
        self.map(Self::Item::u_map)
            .reduce(Self::Item::u_reduce)
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }

    // early exit

    /// Returns the first (or any) element of the iterator.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if a None element is observed first.
    ///
    /// * first element is returned if default iteration order `IterationOrder::Ordered` is used,
    /// * any element is returned if `IterationOrder::Arbitrary` is set.
    ///
    /// See the details here: [crate::ParIter::first].
    fn first(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Send;

    /// Returns the first (or any) element of the iterator that satisfies the `predicate`.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if a None element is observed first.
    ///
    /// * first element is returned if default iteration order `IterationOrder::Ordered` is used,
    /// * any element is returned if `IterationOrder::Arbitrary` is set.
    ///
    /// Note that `find` itself is short-circuiting in addition to fallible computation.
    /// Therefore, in case the fallible iterator contains both a None and a Some element,
    /// the result is **not deterministic**:
    /// * it might be the `None` if it is observed first;
    /// * or `Some(element)` if the Some element satisfying the predicate is observed first.
    ///
    /// Unlike [crate::ParIterOption::find], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn find<Predicate>(self, predicate: Predicate) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send,
        Predicate: Fn(&mut U::Item, &Self::Item) -> bool + Sync,
    {
        self.filter(&predicate).first()
    }
}
