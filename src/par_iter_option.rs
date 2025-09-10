use crate::computations::{map_count, reduce_sum, reduce_unit};
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, Sum};
use core::cmp::Ordering;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with None.
///
/// # Examples
///
/// To demonstrate the difference of fallible iterator's behavior, consider the following simple example.
/// We parse a series of strings into integers.
/// We try this twice:
/// * in the first one, all inputs are good, hence, we obtain Some of parsed numbers,
/// * in the second one, the value in the middle is faulty, we expect the computation to fail.
///
/// In the following, we try to achieve this both with a regular parallel iterator ([`ParIter`]) and a fallible
/// parallel iterator, `ParIterOption` in this case.
///
/// You may notice the following differences:
/// * In the regular iterator, it is not very convenient to keep both the resulting numbers and a potential error.
///   Here, we make use of `filter_map`.
/// * On the other hand, the `collect` method of the fallible iterator directly returns an `Option` of the computation
///   which is either Some of all parsed numbers or None if any computation fails.
/// * Also importantly note that the regular iterator will try to parse all the strings, regardless of how many times
///   the parsing fails.
/// * Fallible iterator, on the other hand, stops immediately after observing the first None and short circuits the
///   computation.
///
/// ```
/// use orx_parallel::*;
///
/// let expected_results = [Some((0..100).collect::<Vec<_>>()), None];
///
/// for expected in expected_results {
///     let expected_some = expected.is_some();
///     let mut inputs: Vec<_> = (0..100).map(|x| x.to_string()).collect();
///     if !expected_some {
///         inputs.insert(50, "x".to_string()); // plant an error case
///     }
///
///     // regular parallel iterator
///     let results = inputs.par().map(|x| x.parse::<u32>().ok());
///     let numbers: Vec<_> = results.filter_map(|x| x).collect();
///     if expected_some {
///         assert_eq!(&expected, &Some(numbers));
///     } else {
///         // otherwise, numbers contains some numbers, but we are not sure
///         // if the computation completely succeeded or not
///     }
///
///     // fallible parallel iterator
///     let results = inputs.par().map(|x| x.parse::<u32>().ok());
///     let result: Option<Vec<_>> = results.into_fallible_option().collect();
///     assert_eq!(&expected, &result);
/// }
/// ```
///
/// These differences are not specific to `collect`; all fallible iterator methods return an option.
/// The following demonstrate reduction examples, where the result is either the reduced value if the entire computation
/// succeeds, or None.
///
/// ```
/// use orx_parallel::*;
///
/// for will_fail in [false, true] {
///     let mut inputs: Vec<_> = (0..100).map(|x| x.to_string()).collect();
///     if will_fail {
///         inputs.insert(50, "x".to_string()); // plant an error case
///     }
///
///     // sum
///     let results = inputs.par().map(|x| x.parse::<u32>().ok());
///     let result: Option<u32> = results.into_fallible_option().sum();
///     match will_fail {
///         true => assert_eq!(result, None),
///         false => assert_eq!(result, Some(4950)),
///     }
///
///     // max
///     let results = inputs.par().map(|x| x.parse::<u32>().ok());
///     let result: Option<Option<u32>> = results.into_fallible_option().max();
///     match will_fail {
///         true => assert_eq!(result, None),
///         false => assert_eq!(result, Some(Some(99))),
///     }
/// }
/// ```
///
/// Finally, similar to regular iterators, a fallible parallel iterator can be tranformed using iterator methods.
/// However, the transformation is on the success path, the failure case of None always short circuits and returns None.
///
/// ```
/// use orx_parallel::*;
///
/// for will_fail in [false, true] {
///     let mut inputs: Vec<_> = (0..100).map(|x| x.to_string()).collect();
///     if will_fail {
///         inputs.insert(50, "x".to_string()); // plant an error case
///     }
///
///     // fallible iter
///     let results = inputs.par().map(|x| x.parse::<u32>().ok());
///     let fallible = results.into_fallible_option();
///
///     // transformations
///
///     let result: Option<usize> = fallible
///         .filter(|x| x % 2 == 1)                                 // Item: u32
///         .map(|x| 3 * x)                                         // Item: u32
///         .filter_map(|x| (x % 10 != 0).then_some(x))             // Item: u32
///         .flat_map(|x| [x.to_string(), (10 * x).to_string()])    // Item: String
///         .map(|x| x.len())                                       // Item: usize
///         .sum();
///
///     match will_fail {
///         true => assert_eq!(result, None),
///         false => assert_eq!(result, Some(312)),
///     }
/// }
/// ```
///
/// [`ParIter`]: crate::ParIter
pub trait ParIterOption<R = DefaultOrchestrator>
where
    R: Orchestrator,
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

    /// Rather than the [`DefaultRunner`], uses the parallel runner `Q` which implements [`Orchestrator`].
    ///
    /// See [`crate::ParIter::with_runner`] for details.
    fn with_runner<Q: Orchestrator>(self) -> impl ParIterOption<Q, Item = Self::Item>;

    // computation transformations

    /// Takes a closure `map` and creates a parallel iterator which calls that closure on each element.
    ///
    /// Transformation is only for the success path where all elements are of the `Some` variant.
    /// Any observation of a `None` case short-circuits the computation and immediately returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    /// let a: Vec<Option<u32>> = vec![Some(1), Some(2), Some(3)];
    /// let iter = a.into_par().into_fallible_option().map(|x| 2 * x);
    ///
    /// let b: Option<Vec<_>> = iter.collect();
    /// assert_eq!(b, Some(vec![2, 4, 6]));
    ///
    /// // at least one fails
    /// let a = vec![Some(1), None, Some(3)];
    /// let iter = a.into_par().into_fallible_option().map(|x| 2 * x);
    ///
    /// let b: Option<Vec<_>> = iter.collect();
    /// assert_eq!(b, None);
    /// ```
    fn map<Out, Map>(self, map: Map) -> impl ParIterOption<R, Item = Out>
    where
        Self: Sized,
        Map: Fn(Self::Item) -> Out + Sync + Clone,
        Out: Send;

    /// Creates an iterator which uses a closure `filter` to determine if an element should be yielded.
    ///
    /// Transformation is only for the success path where all elements are of the `Some` variant.
    /// Any observation of a `None` case short-circuits the computation and immediately returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    /// let a: Vec<Option<i32>> = vec![Some(1), Some(2), Some(3)];
    /// let iter = a.into_par().into_fallible_option().filter(|x| x % 2 == 1);
    ///
    /// let b = iter.sum();
    /// assert_eq!(b, Some(1 + 3));
    ///
    /// // at least one fails
    /// let a = vec![Some(1), None, Some(3)];
    /// let iter = a.into_par().into_fallible_option().filter(|x| x % 2 == 1);
    ///
    /// let b = iter.sum();
    /// assert_eq!(b, None);
    /// ```
    fn filter<Filter>(self, filter: Filter) -> impl ParIterOption<R, Item = Self::Item>
    where
        Self: Sized,
        Filter: Fn(&Self::Item) -> bool + Sync + Clone,
        Self::Item: Send;

    /// Creates an iterator that works like map, but flattens nested structure.
    ///
    /// Transformation is only for the success path where all elements are of the `Some` variant.
    /// Any observation of a `None` case short-circuits the computation and immediately returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    /// let words: Vec<Option<&str>> = vec![Some("alpha"), Some("beta"), Some("gamma")];
    ///
    /// let all_chars: Option<Vec<_>> = words
    ///     .into_par()
    ///     .into_fallible_option()
    ///     .flat_map(|s| s.chars()) // chars() returns an iterator
    ///     .collect();
    ///
    /// let merged: Option<String> = all_chars.map(|chars| chars.iter().collect());
    /// assert_eq!(merged, Some("alphabetagamma".to_string()));
    ///
    /// // at least one fails
    /// let words: Vec<Option<&str>> = vec![Some("alpha"), Some("beta"), None, Some("gamma")];
    ///
    /// let all_chars: Option<Vec<_>> = words
    ///     .into_par()
    ///     .into_fallible_option()
    ///     .flat_map(|s| s.chars()) // chars() returns an iterator
    ///     .collect();
    ///
    /// let merged: Option<String> = all_chars.map(|chars| chars.iter().collect());
    /// assert_eq!(merged, None);
    /// ```
    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIterOption<R, Item = IOut::Item>
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(Self::Item) -> IOut + Sync + Clone;

    /// Creates an iterator that both filters and maps.
    ///
    /// The returned iterator yields only the values for which the supplied closure `filter_map` returns `Some(value)`.
    ///
    /// `filter_map` can be used to make chains of `filter` and `map` more concise.
    /// The example below shows how a `map().filter().map()` can be shortened to a single call to `filter_map`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    /// let a: Vec<Option<&str>> = vec![Some("1"), Some("two"), Some("NaN"), Some("four"), Some("5")];
    ///
    /// let numbers: Option<Vec<_>> = a
    ///     .into_par()
    ///     .into_fallible_option()
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(numbers, Some(vec![1, 5]));
    ///
    /// // at least one fails
    /// let a: Vec<Option<&str>> = vec![Some("1"), Some("two"), None, Some("four"), Some("5")];
    ///
    /// let numbers: Option<Vec<_>> = a
    ///     .into_par()
    ///     .into_fallible_option()
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(numbers, None);
    /// ```
    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIterOption<R, Item = Out>
    where
        Self: Sized,
        FilterMap: Fn(Self::Item) -> Option<Out> + Sync + Clone,
        Out: Send;

    /// Does something with each successful element of an iterator, passing the value on, provided that all elements are of Some variant;
    /// short-circuits and returns None otherwise.
    ///
    /// When using iterators, you’ll often chain several of them together.
    /// While working on such code, you might want to check out what’s happening at various parts in the pipeline.
    /// To do that, insert a call to `inspect()`.
    ///
    /// It’s more common for `inspect()` to be used as a debugging tool than to exist in your final code,
    /// but applications may find it useful in certain situations when errors need to be logged before being discarded.
    ///
    /// It is often convenient to use thread-safe collections such as [`ConcurrentBag`] and
    /// [`ConcurrentVec`](https://crates.io/crates/orx-concurrent-vec) to
    /// collect some intermediate values during parallel execution for further inspection.
    /// The following example demonstrates such a use case.
    ///
    /// [`ConcurrentBag`]: orx_concurrent_bag::ConcurrentBag
    ///
    /// ```
    /// use orx_parallel::*;
    /// use orx_concurrent_bag::*;
    /// use std::num::ParseIntError;
    ///
    /// // all succeeds
    /// let a: Vec<Option<u32>> = ["1", "4", "2", "3"]
    ///     .into_iter()
    ///     .map(|x| x.parse::<u32>().ok())
    ///     .collect();
    ///
    /// // let's add some inspect() calls to investigate what's happening
    /// // - log some events
    /// // - use a concurrent bag to collect and investigate numbers contributing to the sum
    /// let bag = ConcurrentBag::new();
    ///
    /// let sum = a
    ///     .par()
    ///     .cloned()
    ///     .into_fallible_option()
    ///     .inspect(|x| println!("about to filter: {x}"))
    ///     .filter(|x| x % 2 == 0)
    ///     .inspect(|x| {
    ///         bag.push(*x);
    ///         println!("made it through filter: {x}");
    ///     })
    ///     .sum();
    /// assert_eq!(sum, Some(4 + 2));
    ///
    /// let mut values_made_through = bag.into_inner();
    /// values_made_through.sort();
    /// assert_eq!(values_made_through, [2, 4]);
    ///
    /// // at least one fails
    /// let a: Vec<Option<u32>> = ["1", "4", "x", "3"]
    ///     .into_iter()
    ///     .map(|x| x.parse::<u32>().ok())
    ///     .collect();
    ///
    /// // let's add some inspect() calls to investigate what's happening
    /// // - log some events
    /// // - use a concurrent bag to collect and investigate numbers contributing to the sum
    /// let bag = ConcurrentBag::new();
    ///
    /// let sum = a
    ///     .par()
    ///     .cloned()
    ///     .into_fallible_option()
    ///     .inspect(|x| println!("about to filter: {x}"))
    ///     .filter(|x| x % 2 == 0)
    ///     .inspect(|x| {
    ///         bag.push(*x);
    ///         println!("made it through filter: {x}");
    ///     })
    ///     .sum();
    /// assert_eq!(sum, None);
    /// ```
    fn inspect<Operation>(self, operation: Operation) -> impl ParIterOption<R, Item = Self::Item>
    where
        Self: Sized,
        Operation: Fn(&Self::Item) + Sync + Clone,
        Self::Item: Send;

    // collect

    /// Collects all the items from an iterator into a collection iff all elements are of Some variant.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// This is useful when you already have a collection and want to add the iterator items to it.
    ///
    /// The collection is passed in as owned value, and returned back with the additional elements.
    ///
    /// All collections implementing [`ParCollectInto`] can be used to collect into.
    ///
    /// [`ParCollectInto`]: crate::ParCollectInto
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let vec: Vec<i32> = vec![0, 1];
    ///
    /// // all succeeds
    /// let result = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .map(|x| x * 10)
    ///     .collect_into(vec);
    /// assert_eq!(result, Some(vec![0, 1, 10, 20, 30]));
    ///
    /// let vec = result.unwrap();
    ///
    /// // at least one fails
    ///
    /// let result = ["1", "x!", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .map(|x| x * 10)
    ///     .collect_into(vec);
    /// assert_eq!(result, None);
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    ///
    /// let result_doubled: Option<Vec<i32>> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .map(|x| x * 2)
    ///     .collect();
    ///
    /// assert_eq!(result_doubled, Some(vec![2, 4, 6]));
    ///
    /// // at least one fails
    ///
    /// let result_doubled: Option<Vec<i32>> = ["1", "x!", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .map(|x| x * 2)
    ///     .collect();
    ///
    /// assert_eq!(result_doubled, None);
    /// ```
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
    /// The `reduce` function is a closure with two arguments: an ‘accumulator’, and an element.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    /// let reduced: Option<Option<u32>> = (1..10)
    ///     .par()
    ///     .map(|x| 100u32.checked_div(x as u32))
    ///     .into_fallible_option()
    ///     .reduce(|acc, e| acc + e);
    /// assert_eq!(reduced, Some(Some(281)));
    ///
    /// // all succeeds - empty iterator
    /// let reduced: Option<Option<u32>> = (1..1)
    ///     .par()
    ///     .map(|x| 100u32.checked_div(x as u32))
    ///     .into_fallible_option()
    ///     .reduce(|acc, e| acc + e);
    /// assert_eq!(reduced, Some(None));
    ///
    /// // at least one fails
    /// let reduced: Option<Option<u32>> = (0..10)
    ///     .par()
    ///     .map(|x| 100u32.checked_div(x as u32))
    ///     .into_fallible_option()
    ///     .reduce(|acc, e| acc + e);
    /// assert_eq!(reduced, None);
    /// ```
    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync;

    /// Tests if every element of the iterator matches a predicate.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// `all` takes a `predicate` that returns true or false.
    /// It applies this closure to each Ok element of the iterator,
    /// and if they all return true, then so does `all`.
    /// If any of them returns false, it returns false.
    ///
    /// Note that `all` computation is itself also short-circuiting; in other words, it will stop processing as soon as it finds a false,
    /// given that no matter what else happens, the result will also be false.
    ///
    /// Therefore, in case the fallible iterator contains both a None element and a Some element which violates the `predicate`,
    /// the result is **not deterministic**. It might be the `None` if it is observed first; or `Some(false)` if the violation is observed first.
    ///
    /// On the other hand, when it returns `Some(true)`, we are certain that all elements are of `Some` variant and all satisfy the `predicate`.
    ///
    /// An empty iterator returns `Some(true)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all Some
    /// let result = vec!["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .all(|x| *x > 0);
    /// assert_eq!(result, Some(true));
    ///
    /// let result = vec!["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .all(|x| *x > 1);
    /// assert_eq!(result, Some(false));
    ///
    /// let result = Vec::<&str>::new()
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .all(|x| *x > 1);
    /// assert_eq!(result, Some(true)); // empty iterator
    ///
    /// // at least one None
    /// let result = vec!["1", "x!", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .all(|x| *x > 0);
    /// assert_eq!(result, None);
    /// ```
    fn all<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        Self: Sized,
        Self::Item: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        let violates = |x: &Self::Item| !predicate(x);
        self.find(violates).map(|x| x.is_none())
    }

    /// Tests if any element of the iterator matches a predicate.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// `any` takes a `predicate` that returns true or false.
    /// It applies this closure to each element of the iterator,
    /// and if any of the elements returns true, then so does `any`.
    /// If all of them return false, it returns false.
    ///
    /// Note that `any` computation is itself also short-circuiting; in other words, it will stop processing as soon as it finds a true,
    /// given that no matter what else happens, the result will also be true.
    ///
    /// Therefore, in case the fallible iterator contains both a None element and a Some element which satisfies the `predicate`,
    /// the result is **not deterministic**. It might be the `None` if it is observed first; or `Some(true)` if element satisfying the predicate
    /// is observed first.
    ///
    /// On the other hand, when it returns `Some(false)`, we are certain that all elements are of `Some` variant and none of them satisfies the `predicate`.
    ///
    /// An empty iterator returns `Some(false)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all Some
    /// let result = vec!["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .any(|x| *x > 1);
    /// assert_eq!(result, Some(true));
    ///
    /// let result = vec!["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .any(|x| *x > 3);
    /// assert_eq!(result, Some(false));
    ///
    /// let result = Vec::<&str>::new()
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .any(|x| *x > 1);
    /// assert_eq!(result, Some(false)); // empty iterator
    ///
    /// // at least one None
    /// let result = vec!["1", "x!", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .any(|x| *x > 5);
    /// assert_eq!(result, None);
    /// ```
    fn any<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        Self: Sized,
        Self::Item: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        self.find(predicate).map(|x| x.is_some())
    }

    /// Consumes the iterator, counting the number of iterations and returning it.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all Some
    /// let result = vec!["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .filter(|x| *x >= 2)
    ///     .count();
    /// assert_eq!(result, Some(2));
    ///
    /// // at least one None
    /// let result = vec!["x!", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .filter(|x| *x >= 2)
    ///     .count();
    /// assert_eq!(result, None);
    /// ```
    fn count(self) -> Option<usize>
    where
        Self: Sized,
    {
        self.map(map_count)
            .reduce(reduce_sum)
            .map(|x| x.unwrap_or(0))
    }

    /// Calls a closure on each element of an iterator, and returns `Ok(())` if all elements succeed.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use orx_parallel::*;
    /// use std::sync::mpsc::channel;
    ///
    /// // all Some
    /// let (tx, rx) = channel();
    /// let result = vec!["0", "1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .map(|x| x * 2 + 1)
    ///     .for_each(move |x| tx.send(x).unwrap());
    ///
    /// assert_eq!(result, Some(()));
    ///
    /// let mut v: Vec<_> = rx.iter().collect();
    /// v.sort(); // order can be mixed, since messages will be sent in parallel
    /// assert_eq!(v, vec![1, 3, 5, 7, 9]);
    ///
    /// // at least one None
    /// let (tx, _rx) = channel();
    /// let result = vec!["0", "1", "2", "x!", "4"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>().ok())
    ///     .into_fallible_option()
    ///     .map(|x| x * 2 + 1)
    ///     .for_each(move |x| tx.send(x).unwrap());
    ///
    /// assert_eq!(result, None);
    /// ```
    fn for_each<Operation>(self, operation: Operation) -> Option<()>
    where
        Self: Sized,
        Operation: Fn(Self::Item) + Sync,
    {
        let map = |x| operation(x);
        self.map(map).reduce(reduce_unit).map(|_| ())
    }

    /// Returns Some of maximum element of an iterator if all elements succeed.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![Some(1), Some(2), Some(3)];
    /// assert_eq!(a.par().copied().into_fallible_option().max(), Some(Some(3)));
    ///
    /// let b: Vec<Option<i32>> = vec![];
    /// assert_eq!(b.par().copied().into_fallible_option().max(), Some(None));
    ///
    /// let c = vec![Some(1), Some(2), None];
    /// assert_eq!(c.par().copied().into_fallible_option().max(), None);
    /// ```
    fn max(self) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    /// Returns the element that gives the maximum value with respect to the specified `compare` function.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Option<i32>> = vec![Some(1), Some(2), Some(3)];
    /// assert_eq!(
    ///     a.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .max_by(|a, b| a.cmp(b)),
    ///     Some(Some(3))
    /// );
    ///
    /// let b: Vec<Option<i32>> = vec![];
    /// assert_eq!(
    ///     b.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .max_by(|a, b| a.cmp(b)),
    ///     Some(None)
    /// );
    ///
    /// let c: Vec<Option<i32>> = vec![Some(1), Some(2), None];
    /// assert_eq!(
    ///     c.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .max_by(|a, b| a.cmp(b)),
    ///     None
    /// );
    /// ```
    fn max_by<Compare>(self, compare: Compare) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the maximum value from the specified function.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Option<i32>> = vec![Some(-1), Some(2), Some(-3)];
    /// assert_eq!(
    ///     a.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .max_by_key(|x| x.abs()),
    ///     Some(Some(-3))
    /// );
    ///
    /// let b: Vec<Option<i32>> = vec![];
    /// assert_eq!(
    ///     b.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .max_by_key(|x| x.abs()),
    ///     Some(None)
    /// );
    ///
    /// let c: Vec<Option<i32>> = vec![Some(1), Some(2), None];
    /// assert_eq!(
    ///     c.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .max_by_key(|x| x.abs()),
    ///     None
    /// );
    /// ```
    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send,
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |x, y| match key(&x).cmp(&key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns Some of minimum element of an iterator if all elements succeed.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![Some(1), Some(2), Some(3)];
    /// assert_eq!(a.par().copied().into_fallible_option().min(), Some(Some(1)));
    ///
    /// let b: Vec<Option<i32>> = vec![];
    /// assert_eq!(b.par().copied().into_fallible_option().min(), Some(None));
    ///
    /// let c = vec![Some(1), Some(2), None];
    /// assert_eq!(c.par().copied().into_fallible_option().min(), None);
    /// ```
    fn min(self) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::min)
    }

    /// Returns the element that gives the minimum value with respect to the specified `compare` function.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Option<i32>> = vec![Some(1), Some(2), Some(3)];
    /// assert_eq!(
    ///     a.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .min_by(|a, b| a.cmp(b)),
    ///     Some(Some(1))
    /// );
    ///
    /// let b: Vec<Option<i32>> = vec![];
    /// assert_eq!(
    ///     b.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .min_by(|a, b| a.cmp(b)),
    ///     Some(None)
    /// );
    ///
    /// let c: Vec<Option<i32>> = vec![Some(1), Some(2), None];
    /// assert_eq!(
    ///     c.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .min_by(|a, b| a.cmp(b)),
    ///     None
    /// );
    /// ```
    fn min_by<Compare>(self, compare: Compare) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the minimum value from the specified function.
    /// If the iterator is empty, `Some(None)` is returned.
    /// Early exits and returns None if any of the elements is None.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Option<i32>> = vec![Some(-1), Some(2), Some(-3)];
    /// assert_eq!(
    ///     a.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .min_by_key(|x| x.abs()),
    ///     Some(Some(-1))
    /// );
    ///
    /// let b: Vec<Option<i32>> = vec![];
    /// assert_eq!(
    ///     b.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .min_by_key(|x| x.abs()),
    ///     Some(None)
    /// );
    ///
    /// let c: Vec<Option<i32>> = vec![Some(1), Some(2), None];
    /// assert_eq!(
    ///     c.par()
    ///         .copied()
    ///         .into_fallible_option()
    ///         .min_by_key(|x| x.abs()),
    ///     None
    /// );
    /// ```
    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send,
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |x, y| match get_key(&x).cmp(&get_key(&y)) {
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
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    /// let reduced: Option<u32> = (1..10)
    ///     .par()
    ///     .map(|x| 100u32.checked_div(x as u32))
    ///     .into_fallible_option()
    ///     .sum();
    /// assert_eq!(reduced, Some(281));
    ///
    /// // all succeeds - empty iterator
    /// let reduced: Option<u32> = (1..1)
    ///     .par()
    ///     .map(|x| 100u32.checked_div(x as u32))
    ///     .into_fallible_option()
    ///     .sum();
    /// assert_eq!(reduced, Some(0));
    ///
    /// // at least one fails
    /// let reduced: Option<u32> = (0..10)
    ///     .par()
    ///     .map(|x| 100u32.checked_div(x as u32))
    ///     .into_fallible_option()
    ///     .sum();
    /// assert_eq!(reduced, None);
    /// ```
    fn sum<Out>(self) -> Option<Out>
    where
        Self: Sized,
        Self::Item: Sum<Out>,
        Out: Send,
    {
        self.map(Self::Item::map)
            .reduce(Self::Item::reduce)
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
    /// Note that `find` itself is short-circuiting in addition to fallible computation.
    /// Therefore, in case the fallible iterator contains both a None and a Some element,
    /// the result is **not deterministic**:
    /// * it might be the `None` if it is observed first;
    /// * or `Some(element)` if the Some element is observed first.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Option<i32>> = vec![];
    /// assert_eq!(a.par().copied().into_fallible_option().first(), Some(None));
    ///
    /// let a: Vec<Option<i32>> = vec![Some(1), Some(2), Some(3)];
    /// assert_eq!(
    ///     a.par().copied().into_fallible_option().first(),
    ///     Some(Some(1))
    /// );
    ///
    /// let a: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
    /// let result = a.par().copied().into_fallible_option().first();
    /// // depends on whichever is observed first in parallel execution
    /// assert!(result == Some(Some(1)) || result == None);
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Option<i32>> = vec![];
    /// assert_eq!(
    ///     a.par().copied().into_fallible_option().find(|x| *x > 2),
    ///     Some(None)
    /// );
    ///
    /// let a: Vec<Option<i32>> = vec![Some(1), Some(2), Some(3)];
    /// assert_eq!(
    ///     a.par().copied().into_fallible_option().find(|x| *x > 2),
    ///     Some(Some(3))
    /// );
    ///
    /// let a: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
    /// let result = a.par().copied().into_fallible_option().find(|x| *x > 2);
    /// // depends on whichever is observed first in parallel execution
    /// assert!(result == Some(Some(3)) || result == None);
    /// ```
    fn find<Predicate>(self, predicate: Predicate) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        self.filter(&predicate).first()
    }
}

pub trait IntoOption<T> {
    fn into_option(self) -> Option<T>;

    fn into_result_with_unit_err(self) -> Result<T, ()>;
}

impl<T> IntoOption<T> for Option<T> {
    #[inline(always)]
    fn into_option(self) -> Option<T> {
        self
    }

    #[inline(always)]
    fn into_result_with_unit_err(self) -> Result<T, ()> {
        match self {
            Some(x) => Ok(x),
            None => Err(()),
        }
    }
}

pub(crate) trait ResultIntoOption<T> {
    fn into_option(self) -> Option<T>;
}

impl<T> ResultIntoOption<T> for Result<T, ()> {
    #[inline(always)]
    fn into_option(self) -> Option<T> {
        self.ok()
    }
}
