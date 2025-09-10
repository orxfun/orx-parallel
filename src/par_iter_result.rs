use crate::computations::{map_count, reduce_sum, reduce_unit};
use crate::orch::{DefaultOrchestrator, Orchestrator};
use crate::{ChunkSize, IterationOrder, NumThreads, Sum};
use crate::{ParCollectInto, ParIter, generic_values::fallible_iterators::ResultOfIter};
use core::cmp::Ordering;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with an error.
///
/// # Examples
///
/// To demonstrate the difference of fallible iterator's behavior, consider the following simple example.
/// We parse a series of strings into integers.
/// We try this twice:
/// * in the first one, all inputs are good, hence, we obtain Ok of parsed numbers,
/// * in the second one, the value in the middle is faulty, we expect the computation to fail.
///
/// In the following, we try to achieve this both with a regular parallel iterator ([`ParIter`]) and a fallible
/// parallel iterator, `ParIterResult` in this case.
///
/// You may notice the following differences:
/// * In the regular iterator, it is not very convenient to keep both the resulting numbers and a potential error.
///   Here, we make use of `filter_map` and simply ignore the error.
/// * On the other hand, the `collect` method of the fallible iterator directly returns a `Result` of the computation
///   which is either Ok of all parsed numbers or the error.
/// * Also importantly note that the regular iterator will try to parse all the strings, regardless of how many times
///   the parsing fails.
/// * Fallible iterator, on the other hand, stops immediately after observing the first error and short circuits the
///   computation.
///
/// ```
/// use orx_parallel::*;
/// use std::num::{IntErrorKind, ParseIntError};
///
/// let expected_results = [
///     Ok((0..100).collect::<Vec<_>>()),
///     Err(IntErrorKind::InvalidDigit),
/// ];
///
/// for expected in expected_results {
///     let expected_ok = expected.is_ok();
///     let mut inputs: Vec<_> = (0..100).map(|x| x.to_string()).collect();
///     if !expected_ok {
///         inputs.insert(50, "x".to_string()); // plant an error case
///     }
///
///     // regular parallel iterator
///     let results = inputs.par().map(|x| x.parse::<u32>());
///     let numbers: Vec<_> = results.filter_map(|x| x.ok()).collect();
///     if expected_ok {
///         assert_eq!(&expected, &Ok(numbers));
///     } else {
///         // we lost the error
///     }
///
///     // fallible parallel iterator
///     let results = inputs.par().map(|x| x.parse::<u32>());
///     let result: Result<Vec<_>, ParseIntError> = results.into_fallible_result().collect();
///     assert_eq!(&expected, &result.map_err(|x| x.kind().clone()));
/// }
/// ```
///
/// These differences are not specific to `collect`; all fallible iterator methods return a result.
/// The following demonstrate reduction examples, where the result is either the reduced value if the entire computation
/// succeeds, or the error.
///
/// ```
/// use orx_parallel::*;
/// use std::num::ParseIntError;
///
/// for will_fail in [false, true] {
///     let mut inputs: Vec<_> = (0..100).map(|x| x.to_string()).collect();
///     if will_fail {
///         inputs.insert(50, "x".to_string()); // plant an error case
///     }
///
///     // sum
///     let results = inputs.par().map(|x| x.parse::<u32>());
///     let result: Result<u32, ParseIntError> = results.into_fallible_result().sum();
///     match will_fail {
///         true => assert!(result.is_err()),
///         false => assert_eq!(result, Ok(4950)),
///     }
///
///     // max
///     let results = inputs.par().map(|x| x.parse::<u32>());
///     let result: Result<Option<u32>, ParseIntError> = results.into_fallible_result().max();
///     match will_fail {
///         true => assert!(result.is_err()),
///         false => assert_eq!(result, Ok(Some(99))),
///     }
/// }
/// ```
///
/// Finally, similar to regular iterators, a fallible parallel iterator can be tranformed using iterator methods.
/// However, the transformation is on the success path, the error case always short circuits and returns the error.
/// Notice in the following example that the success type keeps changing through transformations while the error type
/// remains the same.
///
/// ```
/// use orx_parallel::*;
/// use std::num::ParseIntError;
///
/// for will_fail in [false, true] {
///     let mut inputs: Vec<_> = (0..100).map(|x| x.to_string()).collect();
///     if will_fail {
///         inputs.insert(50, "x".to_string()); // plant an error case
///     }
///
///     // fallible iter
///     let results = inputs.par().map(|x| x.parse::<u32>());
///     let fallible = results.into_fallible_result();              // Ok: u32, Error: ParseIntError
///
///     // transformations
///
///     let result: Result<usize, ParseIntError> = fallible
///         .filter(|x| x % 2 == 1)                                 // Ok: u32, Error: ParseIntError
///         .map(|x| 3 * x)                                         // Ok: u32, Error: ParseIntError
///         .filter_map(|x| (x % 10 != 0).then_some(x))             // Ok: u32, Error: ParseIntError
///         .flat_map(|x| [x.to_string(), (10 * x).to_string()])    // Ok: String, Error: ParseIntError
///         .map(|x| x.len())                                       // Ok: usize, Error: ParseIntError
///         .sum();
///
///     match will_fail {
///         true => assert!(result.is_err()),
///         false => assert_eq!(result, Ok(312)),
///     }
/// }
/// ```
///
/// [`ParIter`]: crate::ParIter
pub trait ParIterResult<R = DefaultOrchestrator>
where
    R: Orchestrator,
{
    /// Type of the Ok element, to be received as the Ok variant iff the entire computation succeeds.
    type Item;

    /// Type of the Err element, to be received if any of the computations fails.
    type Err;

    /// Element type of the regular parallel iterator this fallible iterator can be converted to, simply `Result<Self::Ok, Self::Err>`.
    type RegularItem: IntoResult<Self::Item, Self::Err>;

    /// Regular parallel iterator this fallible iterator can be converted into.
    type RegularParIter: ParIter<R, Item = Self::RegularItem>;

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

    /// Rather than the [`DefaultRunner`], uses the parallel runner `Q` which implements [`Orchestrator`].
    ///
    /// See [`ParIter::with_runner`] for details.
    fn with_runner<Q: Orchestrator>(
        self,
        orchestrator: Q,
    ) -> impl ParIterResult<Q, Item = Self::Item, Err = Self::Err>;

    // computation transformations

    /// Takes a closure `map` and creates a parallel iterator which calls that closure on each element.
    ///
    /// Transformation is only for the success path where all elements are of the `Ok` variant.
    /// Any observation of an `Err` case short-circuits the computation and immediately returns the observed error.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    /// let a: Vec<Result<u32, char>> = vec![Ok(1), Ok(2), Ok(3)];
    /// let iter = a.into_par().into_fallible_result().map(|x| 2 * x);
    ///
    /// let b: Result<Vec<_>, _> = iter.collect();
    /// assert_eq!(b, Ok(vec![2, 4, 6]));
    ///
    /// // at least one fails
    /// let a = vec![Ok(1), Err('x'), Ok(3)];
    /// let iter = a.into_par().into_fallible_result().map(|x| 2 * x);
    ///
    /// let b: Result<Vec<_>, _> = iter.collect();
    /// assert_eq!(b, Err('x'));
    /// ```
    fn map<Out, Map>(self, map: Map) -> impl ParIterResult<R, Item = Out, Err = Self::Err>
    where
        Self: Sized,
        Map: Fn(Self::Item) -> Out + Sync + Clone,
        Out: Send,
    {
        let par = self.into_regular_par();
        let map = par.map(move |x| x.into_result().map(map.clone()));
        map.into_fallible_result()
    }

    /// Creates an iterator which uses a closure `filter` to determine if an element should be yielded.
    ///
    /// Transformation is only for the success path where all elements are of the `Ok` variant.
    /// Any observation of an `Err` case short-circuits the computation and immediately returns the observed error.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    /// let a: Vec<Result<u32, char>> = vec![Ok(1), Ok(2), Ok(3)];
    /// let iter = a.into_par().into_fallible_result().filter(|x| x % 2 == 1);
    ///
    /// let b = iter.sum();
    /// assert_eq!(b, Ok(1 + 3));
    ///
    /// // at least one fails
    /// let a = vec![Ok(1), Err('x'), Ok(3)];
    /// let iter = a.into_par().into_fallible_result().filter(|x| x % 2 == 1);
    ///
    /// let b = iter.sum();
    /// assert_eq!(b, Err('x'));
    /// ```
    fn filter<Filter>(
        self,
        filter: Filter,
    ) -> impl ParIterResult<R, Item = Self::Item, Err = Self::Err>
    where
        Self: Sized,
        Filter: Fn(&Self::Item) -> bool + Sync + Clone,
        Self::Item: Send,
    {
        let par = self.into_regular_par();
        let filter_map = par.filter_map(move |x| match x.into_result() {
            Ok(x) => match filter(&x) {
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
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    /// let words: Vec<Result<&str, char>> = vec![Ok("alpha"), Ok("beta"), Ok("gamma")];
    ///
    /// let all_chars: Result<Vec<_>, _> = words
    ///     .into_par()
    ///     .into_fallible_result()
    ///     .flat_map(|s| s.chars()) // chars() returns an iterator
    ///     .collect();
    ///
    /// let merged: Result<String, _> = all_chars.map(|chars| chars.iter().collect());
    /// assert_eq!(merged, Ok("alphabetagamma".to_string()));
    ///
    /// // at least one fails
    /// let words: Vec<Result<&str, char>> = vec![Ok("alpha"), Ok("beta"), Err('x'), Ok("gamma")];
    ///
    /// let all_chars: Result<Vec<_>, _> = words
    ///     .into_par()
    ///     .into_fallible_result()
    ///     .flat_map(|s| s.chars()) // chars() returns an iterator
    ///     .collect();
    ///
    /// let merged: Result<String, _> = all_chars.map(|chars| chars.iter().collect());
    /// assert_eq!(merged, Err('x'));
    /// ```
    fn flat_map<IOut, FlatMap>(
        self,
        flat_map: FlatMap,
    ) -> impl ParIterResult<R, Item = IOut::Item, Err = Self::Err>
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(Self::Item) -> IOut + Sync + Clone,
    {
        let par = self.into_regular_par();
        let map = par.flat_map(move |x| match x.into_result() {
            Ok(x) => ResultOfIter::ok(flat_map(x).into_iter()),
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
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    /// let a: Vec<Result<&str, char>> = vec![Ok("1"), Ok("two"), Ok("NaN"), Ok("four"), Ok("5")];
    ///
    /// let numbers: Result<Vec<_>, char> = a
    ///     .into_par()
    ///     .into_fallible_result()
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(numbers, Ok(vec![1, 5]));
    ///
    /// // at least one fails
    /// let a: Vec<Result<&str, char>> = vec![Ok("1"), Ok("two"), Err('x'), Ok("four"), Ok("5")];
    ///
    /// let numbers: Result<Vec<_>, char> = a
    ///     .into_par()
    ///     .into_fallible_result()
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(numbers, Err('x'));
    /// ```
    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterResult<R, Item = Out, Err = Self::Err>
    where
        Self: Sized,
        FilterMap: Fn(Self::Item) -> Option<Out> + Sync + Clone,
        Out: Send,
    {
        let par = self.into_regular_par();
        let filter_map = par.filter_map(move |x| match x.into_result() {
            Ok(x) => filter_map(x).map(|x| Ok(x)),
            Err(e) => Some(Err(e)),
        });
        filter_map.into_fallible_result()
    }

    /// Does something with each successful element of an iterator, passing the value on, provided that all elements are of Ok variant;
    /// short-circuits and returns the error otherwise.
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
    /// let a: Vec<Result<u32, ParseIntError>> = ["1", "4", "2", "3"]
    ///     .into_iter()
    ///     .map(|x| x.parse::<u32>())
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
    ///     .into_fallible_result()
    ///     .inspect(|x| println!("about to filter: {x}"))
    ///     .filter(|x| x % 2 == 0)
    ///     .inspect(|x| {
    ///         bag.push(*x);
    ///         println!("made it through filter: {x}");
    ///     })
    ///     .sum();
    /// assert_eq!(sum, Ok(4 + 2));
    ///
    /// let mut values_made_through = bag.into_inner();
    /// values_made_through.sort();
    /// assert_eq!(values_made_through, [2, 4]);
    ///
    /// // at least one fails
    /// let a: Vec<Result<u32, ParseIntError>> = ["1", "4", "x", "3"]
    ///     .into_iter()
    ///     .map(|x| x.parse::<u32>())
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
    ///     .into_fallible_result()
    ///     .inspect(|x| println!("about to filter: {x}"))
    ///     .filter(|x| x % 2 == 0)
    ///     .inspect(|x| {
    ///         bag.push(*x);
    ///         println!("made it through filter: {x}");
    ///     })
    ///     .sum();
    /// assert!(sum.is_err());
    /// ```
    fn inspect<Operation>(
        self,
        operation: Operation,
    ) -> impl ParIterResult<R, Item = Self::Item, Err = Self::Err>
    where
        Self: Sized,
        Operation: Fn(&Self::Item) + Sync + Clone,
        Self::Item: Send,
    {
        let map = move |x| {
            operation(&x);
            x
        };
        self.map(map)
    }

    // collect

    /// Collects all the items from an iterator into a collection iff all elements are of Ok variant.
    /// Early exits and returns the error if any of the elements is an Err.
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
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .map(|x| x * 10)
    ///     .collect_into(vec);
    ///
    /// assert!(result.is_ok());
    /// let vec = result.unwrap();
    /// assert_eq!(vec, vec![0, 1, 10, 20, 30]);
    ///
    /// // at least one fails
    ///
    /// let result = ["1", "x!", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .map(|x| x * 10)
    ///     .collect_into(vec);
    /// assert!(result.is_err());
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all succeeds
    ///
    /// let result_doubled: Result<Vec<i32>, _> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .map(|x| x * 2)
    ///     .collect();
    ///
    /// assert_eq!(result_doubled, Ok(vec![2, 4, 6]));
    ///
    /// // at least one fails
    ///
    /// let result_doubled: Result<Vec<i32>, _> = ["1", "x!", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .map(|x| x * 2)
    ///     .collect();
    ///
    /// assert!(result_doubled.is_err());
    /// ```
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
    /// The `reduce` function is a closure with two arguments: an ‘accumulator’, and an element.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// fn safe_div(a: u32, b: u32) -> Result<u32, char> {
    ///     match b {
    ///         0 => Err('!'),
    ///         b => Ok(a / b),
    ///     }
    /// }
    ///
    /// // all succeeds
    /// let reduced: Result<Option<u32>, char> = (1..10)
    ///     .par()
    ///     .map(|x| safe_div(100, x as u32))
    ///     .into_fallible_result()
    ///     .reduce(|acc, e| acc + e);
    /// assert_eq!(reduced, Ok(Some(281)));
    ///
    /// // all succeeds - empty iterator
    /// let reduced: Result<Option<u32>, char> = (1..1)
    ///     .par()
    ///     .map(|x| safe_div(100, x as u32))
    ///     .into_fallible_result()
    ///     .reduce(|acc, e| acc + e);
    /// assert_eq!(reduced, Ok(None));
    ///
    /// // at least one fails
    /// let reduced: Result<Option<u32>, char> = (0..10)
    ///     .par()
    ///     .map(|x| safe_div(100, x as u32))
    ///     .into_fallible_result()
    ///     .reduce(|acc, e| acc + e);
    /// assert_eq!(reduced, Err('!'));
    /// ```
    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync;

    /// Tests if every element of the iterator matches a predicate.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// `all` takes a `predicate` that returns true or false.
    /// It applies this closure to each Ok element of the iterator,
    /// and if they all return true, then so does `all`.
    /// If any of them returns false, it returns false.
    ///
    /// Note that `all` computation is itself also short-circuiting; in other words, it will stop processing as soon as it finds a false,
    /// given that no matter what else happens, the result will also be false.
    ///
    /// Therefore, in case the fallible iterator contains both an Err element and an Ok element which violates the `predicate`,
    /// the result is **not deterministic**. It might be the `Err` if it is observed first; or `Ok(false)` if the violation is observed first.
    ///
    /// On the other hand, when it returns `Ok(true)`, we are certain that all elements are of `Ok` variant and all satisfy the `predicate`.
    ///
    /// An empty iterator returns `Ok(true)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all Ok
    /// let result = vec!["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .all(|x| *x > 0);
    /// assert_eq!(result, Ok(true));
    ///
    /// let result = vec!["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .all(|x| *x > 1);
    /// assert_eq!(result, Ok(false));
    ///
    /// let result = Vec::<&str>::new()
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .all(|x| *x > 1);
    /// assert_eq!(result, Ok(true)); // empty iterator
    ///
    /// // at least one Err
    /// let result = vec!["1", "x!", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .all(|x| *x > 0);
    /// assert!(result.is_err());
    /// ```
    fn all<Predicate>(self, predicate: Predicate) -> Result<bool, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        let violates = |x: &Self::Item| !predicate(x);
        self.find(violates).map(|x| x.is_none())
    }

    /// Tests if any element of the iterator matches a predicate.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// `any` takes a `predicate` that returns true or false.
    /// It applies this closure to each element of the iterator,
    /// and if any of the elements returns true, then so does `any`.
    /// If all of them return false, it returns false.
    ///
    /// Note that `any` computation is itself also short-circuiting; in other words, it will stop processing as soon as it finds a true,
    /// given that no matter what else happens, the result will also be true.
    ///
    /// Therefore, in case the fallible iterator contains both an Err element and an Ok element which satisfies the `predicate`,
    /// the result is **not deterministic**. It might be the `Err` if it is observed first; or `Ok(true)` if element satisfying the predicate
    /// is observed first.
    ///
    /// On the other hand, when it returns `Ok(false)`, we are certain that all elements are of `Ok` variant and none of them satisfies the `predicate`.
    ///
    /// An empty iterator returns `Ok(false)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all Ok
    /// let result = vec!["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .any(|x| *x > 1);
    /// assert_eq!(result, Ok(true));
    ///
    /// let result = vec!["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .any(|x| *x > 3);
    /// assert_eq!(result, Ok(false));
    ///
    /// let result = Vec::<&str>::new()
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .any(|x| *x > 1);
    /// assert_eq!(result, Ok(false)); // empty iterator
    ///
    /// // at least one Err
    /// let result = vec!["1", "x!", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .any(|x| *x > 5);
    /// assert!(result.is_err());
    /// ```
    fn any<Predicate>(self, predicate: Predicate) -> Result<bool, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        self.find(predicate).map(|x| x.is_some())
    }

    /// Consumes the iterator, counting the number of iterations and returning it.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // all Ok
    /// let result = vec!["1", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .filter(|x| *x >= 2)
    ///     .count();
    /// assert_eq!(result, Ok(2));
    ///
    /// // at least one Err
    /// let result = vec!["x!", "2", "3"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .filter(|x| *x >= 2)
    ///     .count();
    /// assert!(result.is_err());
    /// ```
    fn count(self) -> Result<usize, Self::Err>
    where
        Self: Sized,
        Self::Err: Send,
    {
        self.map(map_count)
            .reduce(reduce_sum)
            .map(|x| x.unwrap_or(0))
    }

    /// Calls a closure on each element of an iterator, and returns `Ok(())` if all elements succeed.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use orx_parallel::*;
    /// use std::sync::mpsc::channel;
    ///
    /// // all Ok
    /// let (tx, rx) = channel();
    /// let result = vec!["0", "1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .map(|x| x * 2 + 1)
    ///     .for_each(move |x| tx.send(x).unwrap());
    ///
    /// assert_eq!(result, Ok(()));
    ///
    /// let mut v: Vec<_> = rx.iter().collect();
    /// v.sort(); // order can be mixed, since messages will be sent in parallel
    /// assert_eq!(v, vec![1, 3, 5, 7, 9]);
    ///
    /// // at least one Err
    /// let (tx, _rx) = channel();
    /// let result = vec!["0", "1", "2", "x!", "4"]
    ///     .into_par()
    ///     .map(|x| x.parse::<i32>())
    ///     .into_fallible_result()
    ///     .map(|x| x * 2 + 1)
    ///     .for_each(move |x| tx.send(x).unwrap());
    ///
    /// assert!(result.is_err());
    /// ```
    fn for_each<Operation>(self, operation: Operation) -> Result<(), Self::Err>
    where
        Self: Sized,
        Self::Err: Send,
        Operation: Fn(Self::Item) + Sync,
    {
        let map = |x| operation(x);
        self.map(map).reduce(reduce_unit).map(|_| ())
    }

    /// Returns Ok of maximum element of an iterator if all elements succeed.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Ok(3)];
    /// assert_eq!(a.par().copied().into_fallible_result().max(), Ok(Some(3)));
    ///
    /// let b: Vec<Result<i32, char>> = vec![];
    /// assert_eq!(b.par().copied().into_fallible_result().max(), Ok(None));
    ///
    /// let c: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Err('x')];
    /// assert_eq!(c.par().copied().into_fallible_result().max(), Err('x'));
    /// ```
    fn max(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Err: Send,
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    /// Returns the element that gives the maximum value with respect to the specified `compare` function.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Ok(3)];
    /// assert_eq!(
    ///     a.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .max_by(|a, b| a.cmp(b)),
    ///     Ok(Some(3))
    /// );
    ///
    /// let b: Vec<Result<i32, char>> = vec![];
    /// assert_eq!(
    ///     b.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .max_by(|a, b| a.cmp(b)),
    ///     Ok(None)
    /// );
    ///
    /// let c: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Err('x')];
    /// assert_eq!(
    ///     c.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .max_by(|a, b| a.cmp(b)),
    ///     Err('x')
    /// );
    /// ```
    fn max_by<Compare>(self, compare: Compare) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the maximum value from the specified function.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Result<i32, char>> = vec![Ok(-1), Ok(2), Ok(-3)];
    /// assert_eq!(
    ///     a.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .max_by_key(|x| x.abs()),
    ///     Ok(Some(-3))
    /// );
    ///
    /// let b: Vec<Result<i32, char>> = vec![];
    /// assert_eq!(
    ///     b.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .max_by_key(|x| x.abs()),
    ///     Ok(None)
    /// );
    ///
    /// let c: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Err('x')];
    /// assert_eq!(
    ///     c.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .max_by_key(|x| x.abs()),
    ///     Err('x')
    /// );
    /// ```
    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |x, y| match key(&x).cmp(&key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns Ok of minimum element of an iterator if all elements succeed.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Ok(3)];
    /// assert_eq!(a.par().copied().into_fallible_result().min(), Ok(Some(1)));
    ///
    /// let b: Vec<Result<i32, char>> = vec![];
    /// assert_eq!(b.par().copied().into_fallible_result().min(), Ok(None));
    ///
    /// let c: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Err('x')];
    /// assert_eq!(c.par().copied().into_fallible_result().min(), Err('x'));
    /// ```
    fn min(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Ord + Send,
        Self::Err: Send,
    {
        self.reduce(Ord::min)
    }

    /// Returns the element that gives the maximum value with respect to the specified `compare` function.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Ok(3)];
    /// assert_eq!(
    ///     a.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .min_by(|a, b| a.cmp(b)),
    ///     Ok(Some(1))
    /// );
    ///
    /// let b: Vec<Result<i32, char>> = vec![];
    /// assert_eq!(
    ///     b.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .min_by(|a, b| a.cmp(b)),
    ///     Ok(None)
    /// );
    ///
    /// let c: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Err('x')];
    /// assert_eq!(
    ///     c.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .min_by(|a, b| a.cmp(b)),
    ///     Err('x')
    /// );
    /// ```
    fn min_by<Compare>(self, compare: Compare) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the minimum value from the specified function.
    /// If the iterator is empty, `Ok(None)` is returned.
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Result<i32, char>> = vec![Ok(-1), Ok(2), Ok(-3)];
    /// assert_eq!(
    ///     a.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .min_by_key(|x| x.abs()),
    ///     Ok(Some(-1))
    /// );
    ///
    /// let b: Vec<Result<i32, char>> = vec![];
    /// assert_eq!(
    ///     b.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .min_by_key(|x| x.abs()),
    ///     Ok(None)
    /// );
    ///
    /// let c: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Err('x')];
    /// assert_eq!(
    ///     c.par()
    ///         .copied()
    ///         .into_fallible_result()
    ///         .min_by_key(|x| x.abs()),
    ///     Err('x')
    /// );
    /// ```
    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
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
    /// Early exits and returns the error if any of the elements is an Err.
    ///
    /// If the iterator is empty, returns zero; otherwise, returns `Ok` of the sum.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// fn safe_div(a: u32, b: u32) -> Result<u32, char> {
    ///     match b {
    ///         0 => Err('!'),
    ///         b => Ok(a / b),
    ///     }
    /// }
    ///
    /// // all succeeds
    /// let reduced: Result<u32, char> = (1..10)
    ///     .par()
    ///     .map(|x| safe_div(100, x as u32))
    ///     .into_fallible_result()
    ///     .sum();
    /// assert_eq!(reduced, Ok(281));
    ///
    /// // all succeeds - empty iterator
    /// let reduced: Result<u32, char> = (1..1)
    ///     .par()
    ///     .map(|x| safe_div(100, x as u32))
    ///     .into_fallible_result()
    ///     .sum();
    /// assert_eq!(reduced, Ok(0));
    ///
    /// // at least one fails
    /// let reduced: Result<u32, char> = (0..10)
    ///     .par()
    ///     .map(|x| safe_div(100, x as u32))
    ///     .into_fallible_result()
    ///     .sum();
    /// assert_eq!(reduced, Err('!'));
    /// ```
    fn sum<Out>(self) -> Result<Out, Self::Err>
    where
        Self: Sized,
        Self::Item: Sum<Out>,
        Self::Err: Send,
        Out: Send,
    {
        self.map(Self::Item::map)
            .reduce(Self::Item::reduce)
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
    /// Note that `find` itself is short-circuiting in addition to fallible computation.
    /// Therefore, in case the fallible iterator contains both an Err and an Ok element,
    /// the result is **not deterministic**:
    /// * it might be the `Err` if it is observed first;
    /// * or `Ok(element)` if the Ok element is observed first.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Result<i32, char>> = vec![];
    /// assert_eq!(a.par().copied().into_fallible_result().first(), Ok(None));
    ///
    /// let a: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Ok(3)];
    /// assert_eq!(a.par().copied().into_fallible_result().first(), Ok(Some(1)));
    ///
    /// let a: Vec<Result<i32, char>> = vec![Ok(1), Err('x'), Ok(3)];
    /// let result = a.par().copied().into_fallible_result().first();
    /// // depends on whichever is observed first in parallel execution
    /// assert!(result == Ok(Some(1)) || result == Err('x'));
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<Result<i32, char>> = vec![];
    /// assert_eq!(
    ///     a.par().copied().into_fallible_result().find(|x| *x > 2),
    ///     Ok(None)
    /// );
    ///
    /// let a: Vec<Result<i32, char>> = vec![Ok(1), Ok(2), Ok(3)];
    /// assert_eq!(
    ///     a.par().copied().into_fallible_result().find(|x| *x > 2),
    ///     Ok(Some(3))
    /// );
    ///
    /// let a: Vec<Result<i32, char>> = vec![Ok(1), Err('x'), Ok(3)];
    /// let result = a.par().copied().into_fallible_result().find(|x| *x > 2);
    /// // depends on whichever is observed first in parallel execution
    /// assert!(result == Ok(Some(3)) || result == Err('x'));
    /// ```
    fn find<Predicate>(self, predicate: Predicate) -> Result<Option<Self::Item>, Self::Err>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Err: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        self.filter(&predicate).first()
    }
}

pub trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl<T, E> IntoResult<T, E> for Result<T, E> {
    #[inline(always)]
    fn into_result(self) -> Result<T, E> {
        self
    }
}
