use crate::default_fns::{u_map_count, u_reduce_sum, u_reduce_unit};
use crate::par_iter_result::IntoResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::using::Using;
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParIterUsing, ParThreadPool, RunnerWithPool, Sum,
};
use crate::{ParCollectInto, generic_values::fallible_iterators::ResultOfIter};
use core::cmp::Ordering;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with an error.
///
/// Unlike [crate::ParIterResult], the threads have access to a mutable reference of the used variable in each thread.
///
/// Please see [`crate::ParIterUsing`] for details and examples.
///
/// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
pub trait ParIterResultUsing<U, R = DefaultRunner>
where
    R: ParallelRunner,
    U: Using,
{
    /// Type of the Ok element, to be received as the Ok variant iff the entire computation succeeds.
    type Item;

    /// Type of the Err element, to be received if any of the computations fails.
    type Err;

    /// Element type of the regular parallel iterator this fallible iterator can be converted to, simply `Result<Self::Ok, Self::Err>`.
    type RegularItem: IntoResult<Self::Item, Self::Err>;

    /// Regular parallel iterator this fallible iterator can be converted into.
    type RegularParIter: ParIterUsing<U, R, Item = Self::RegularItem>;

    /// Returns a reference to the input concurrent iterator.
    fn con_iter_len(&self) -> Option<usize>;

    /// Converts this fallible iterator into a regular parallel iterator; i.e., [`ParIter`], with `Item = Result<Self::Ok, Self::Err>`.
    fn into_regular_par(self) -> Self::RegularParIter;

    /// Converts the `regular_par` iterator with `Item = Result<Self::Ok, Self::Err>` into fallible result iterator.
    fn from_regular_par(regular_par: Self::RegularParIter) -> Self;

    // params transformations

    /// Sets the number of threads to be used in the parallel execution.
    /// Integers can be used as the argument with the following mapping:
    ///
    /// * `0` -> `NumThreads::Auto`
    /// * `1` -> `NumThreads::sequential()`
    /// * `n > 0` -> `NumThreads::Max(n)`
    ///
    /// See [`NumThreads`] and [`ParIter::num_threads`] for details.
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self
    where
        Self: Sized,
    {
        Self::from_regular_par(self.into_regular_par().num_threads(num_threads))
    }

    /// Sets the number of elements to be pulled from the concurrent iterator during the
    /// parallel execution. When integers are used as argument, the following mapping applies:
    ///
    /// * `0` -> `ChunkSize::Auto`
    /// * `n > 0` -> `ChunkSize::Exact(n)`
    ///
    /// Please use the default enum constructor for creating `ChunkSize::Min` variant.
    ///
    /// See [`ChunkSize`] and [`ParIter::chunk_size`] for details.
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self
    where
        Self: Sized,
    {
        Self::from_regular_par(self.into_regular_par().chunk_size(chunk_size))
    }

    /// Sets the iteration order of the parallel computation.
    ///
    /// See [`IterationOrder`] and [`ParIter::iteration_order`] for details.
    fn iteration_order(self, order: IterationOrder) -> Self
    where
        Self: Sized,
    {
        Self::from_regular_par(self.into_regular_par().iteration_order(order))
    }

    /// Rather than the [`DefaultRunner`], uses the parallel runner `Q` which implements [`ParallelRunner`].
    ///
    /// See [`ParIter::with_runner`] for details.
    ///
    /// [`DefaultRunner`]: crate::DefaultRunner
    /// [`ParIter::with_runner`]: crate::ParIter::with_runner
    fn with_runner<Q: ParallelRunner>(
        self,
        orchestrator: Q,
    ) -> impl ParIterResultUsing<U, Q, Item = Self::Item, Err = Self::Err>;

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
    ) -> impl ParIterResultUsing<U, RunnerWithPool<P, R::Executor>, Item = Self::Item, Err = Self::Err>
    where
        Self: Sized,
    {
        let runner = RunnerWithPool::from(pool).with_executor::<R::Executor>();
        self.with_runner(runner)
    }

    // computation transformations

    /// Takes a closure `map` and creates a parallel iterator which calls that closure on each element.
    ///
    /// Transformation is only for the success path where all elements are of the `Ok` variant.
    /// Any observation of an `Err` case short-circuits the computation and immediately returns the observed error.
    ///
    /// Unlike [crate::ParIterResult::map], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn map<Out, Map>(self, map: Map) -> impl ParIterResultUsing<U, R, Item = Out, Err = Self::Err>
    where
        Self: Sized,
        Map: Fn(&mut U::Item, Self::Item) -> Out + Sync + Clone,
        Out: Send,
    {
        let par = self.into_regular_par();
        let map = par.map(move |u, x| x.into_result().map(|inner| map.clone()(u, inner)));
        map.into_fallible_result()
    }

    /// Creates an iterator which uses a closure `filter` to determine if an element should be yielded.
    ///
    /// Transformation is only for the success path where all elements are of the `Ok` variant.
    /// Any observation of an `Err` case short-circuits the computation and immediately returns the observed error.
    ///
    /// Unlike [crate::ParIterResult::filter], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn filter<Filter>(
        self,
        filter: Filter,
    ) -> impl ParIterResultUsing<U, R, Item = Self::Item, Err = Self::Err>
    where
        Self: Sized,
        Filter: Fn(&mut U::Item, &Self::Item) -> bool + Sync + Clone,
        Self::Item: Send,
    {
        let par = self.into_regular_par();
        let filter_map = par.filter_map(move |u, x| match x.into_result() {
            Ok(x) => match filter(u, &x) {
                true => Some(Ok(x)),
                false => None,
            },
            Err(e) => Some(Err(e)),
        });
        filter_map.into_fallible_result()
    }

    /// Creates an iterator that works like map, but flattens nested structure.
    ///
    /// Transformation is only for the success path where all elements are of the `Ok` variant.
    /// Any observation of an `Err` case short-circuits the computation and immediately returns the observed error.
    ///
    /// Unlike [crate::ParIterResult::flat_map], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterResultUsing<U, R, Item = IOut::Item, Err = Self::Err>
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(&mut U::Item, Self::Item) -> IOut + Sync + Clone,
    {
        let par = self.into_regular_par();
        let map = par.flat_map(move |u, x| match x.into_result() {
            Ok(x) => ResultOfIter::ok(flat_map(u, x).into_iter()),
            Err(e) => ResultOfIter::err(e),
        });
        map.into_fallible_result()
    }

    /// Creates an iterator that both filters and maps.
    ///
    /// The returned iterator yields only the values for which the supplied closure `filter_map` returns `Some(value)`.
    ///
    /// `filter_map` can be used to make chains of `filter` and `map` more concise.
    /// The example below shows how a `map().filter().map()` can be shortened to a single call to `filter_map`.
    ///
    /// Unlike [crate::ParIterResult::filter_map], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterResultUsing<U, R, Item = Out, Err = Self::Err>
    where
        Self: Sized,
        FilterMap: Fn(&mut U::Item, Self::Item) -> Option<Out> + Sync + Clone,
        Out: Send,
    {
        let par = self.into_regular_par();
        let filter_map = par.filter_map(move |u, x| match x.into_result() {
            Ok(x) => filter_map(u, x).map(|x| Ok(x)),
            Err(e) => Some(Err(e)),
        });
        filter_map.into_fallible_result()
    }

    /// Does something with each successful element of an iterator, passing the value on, provided that all elements are of Ok variant;
    /// short-circuits and returns the error otherwise.
    ///
    /// Unlike [crate::ParIterResult::inspect], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn inspect<Operation>(
        self,
        operation: Operation,
    ) -> impl ParIterResultUsing<U, R, Item = Self::Item, Err = Self::Err>
    where
        Self: Sized,
        Operation: Fn(&mut U::Item, &Self::Item) + Sync + Clone,
        Self::Item: Send,
    {
        let map = move |u: &mut U::Item, x: Self::Item| {
            operation(u, &x);
            x
        };
        self.map(map)
    }

    // collect

    /// Collects all the items from an iterator into a collection iff all elements are of Ok variant.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// Unlike [crate::ParIterResult::collect_into], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn collect_into<C>(self, output: C) -> Result<C, Self::Err>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
        Self::Err: Send;

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
    /// Unlike [crate::ParIterResult::collect], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn collect<C>(self) -> Result<C, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        C: ParCollectInto<Self::Item>,
    {
        let output = C::empty(self.con_iter_len());
        self.collect_into(output)
    }

    // reduce

    /// Reduces the elements to a single one, by repeatedly applying a reducing operation.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// If the iterator is empty, returns `Ok(None)`; otherwise, returns `Ok` of result of the reduction.
    ///
    /// Unlike [crate::ParIterResult::reduce], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync;

    /// Tests if every element of the iterator matches a predicate.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// Unlike [crate::ParIterResult::all], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn all<Predicate>(self, predicate: Predicate) -> Result<bool, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Predicate: Fn(&mut U::Item, &Self::Item) -> bool + Sync,
    {
        let violates = |u: &mut U::Item, x: &Self::Item| !predicate(u, x);
        self.find(violates).map(|x| x.is_none())
    }

    /// Tests if any element of the iterator matches a predicate.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// Unlike [crate::ParIterResult::any], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn any<Predicate>(self, predicate: Predicate) -> Result<bool, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Predicate: Fn(&mut U::Item, &Self::Item) -> bool + Sync,
    {
        self.find(predicate).map(|x| x.is_some())
    }

    /// Consumes the iterator, counting the number of iterations and returning it.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// See the details here: [crate::ParIterResult::count].
    fn count(self) -> Result<usize, Self::Err>
    where
        Self: Sized,
        Self::Err: Send,
    {
        self.map(u_map_count)
            .reduce(u_reduce_sum)
            .map(|x| x.unwrap_or(0))
    }

    /// Calls a closure on each element of an iterator, and returns `Ok(())` if all elements succeed.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// Unlike [crate::ParIterResult::for_each], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn for_each<Operation>(self, operation: Operation) -> Result<(), Self::Err>
    where
        Self: Sized,
        Self::Err: Send,
        Operation: Fn(&mut U::Item, Self::Item) + Sync,
    {
        let map = |u: &mut U::Item, x: Self::Item| operation(u, x);
        self.map(map).reduce(u_reduce_unit).map(|_| ())
    }

    /// Returns Ok of maximum element of an iterator if all elements succeed.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// See the details here: [crate::ParIterResult::max].
    fn max(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Err: Send,
        Self::Item: Ord + Send,
    {
        self.reduce(|_, a, b| Ord::max(a, b))
    }

    /// Returns the element that gives the maximum value with respect to the specified `compare` function.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// See the details here: [crate::ParIterResult::max_by].
    fn max_by<Compare>(self, compare: Compare) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |_: &mut U::Item, x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the maximum value from the specified function.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// See the details here: [crate::ParIterResult::max_by_key].
    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |_: &mut U::Item, x, y| match key(&x).cmp(&key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns Ok of minimum element of an iterator if all elements succeed.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// See the details here: [crate::ParIterResult::min].
    fn min(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Ord + Send,
        Self::Err: Send,
    {
        self.reduce(|_, a, b| Ord::min(a, b))
    }

    /// Returns the element that gives the maximum value with respect to the specified `compare` function.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// See the details here: [crate::ParIterResult::min_by].
    fn min_by<Compare>(self, compare: Compare) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |_: &mut U::Item, x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the minimum value from the specified function.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// See the details here: [crate::ParIterResult::min_by_key].
    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
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
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// If the iterator is empty, returns zero; otherwise, returns `Ok` of the sum.
    ///
    /// See the details here: [crate::ParIterResult::sum].
    fn sum<Out>(self) -> Result<Out, Self::Err>
    where
        Self: Sized,
        Self::Item: Sum<Out>,
        Self::Err: Send,
        Out: Send,
    {
        self.map(Self::Item::u_map)
            .reduce(Self::Item::u_reduce)
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }

    // early exit

    /// Returns the first (or any) element of the iterator.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if an Err element is observed first.
    ///
    /// * first element is returned if default iteration order `IterationOrder::Ordered` is used,
    /// * any element is returned if `IterationOrder::Arbitrary` is set.
    ///
    /// See the details here: [crate::ParIter::first].
    fn first(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send;

    /// Returns the first (or any) element of the iterator that satisfies the `predicate`.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if an Err element is observed first.
    ///
    /// * first element is returned if default iteration order `IterationOrder::Ordered` is used,
    /// * any element is returned if `IterationOrder::Arbitrary` is set.
    ///
    /// Note that `find` itself is short-circuiting in addition to fallible computation.
    /// Therefore, in case the fallible iterator contains both an Err and an Ok element,
    /// the result is **not deterministic**:
    /// * it might be the `Err` if it is observed first;
    /// * or `Ok(element)` if the Ok element satisfying the predicate is observed first.
    ///
    /// Unlike [crate::ParIterResult::find], the closure allows access to mutable reference of the used variable.
    ///
    /// Please see [`crate::ParIter::using`] transformation for details and examples.
    ///
    /// Further documentation can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    fn find<Predicate>(self, predicate: Predicate) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Predicate: Fn(&mut U::Item, &Self::Item) -> bool + Sync,
    {
        self.filter(&predicate).first()
    }
}
