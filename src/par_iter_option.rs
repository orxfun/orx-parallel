use crate::computations::{map_count, reduce_sum, reduce_unit};
use crate::{
    ChunkSize, DefaultRunner, IterationOrder, NumThreads, ParCollectInto, ParallelRunner, Sum,
};
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
pub trait ParIterOption<R = DefaultRunner>
where
    R: ParallelRunner,
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
    /// See [`NumThreads`] and [`ParIter::num_threads`] for details.
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets the number of elements to be pulled from the concurrent iterator during the
    /// parallel execution. When integers are used as argument, the following mapping applies:
    ///
    /// * `0` -> `ChunkSize::Auto`
    /// * `n > 0` -> `ChunkSize::Exact(n)`
    ///
    /// Please use the default enum constructor for creating `ChunkSize::Min` variant.
    ///
    /// See [`ChunkSize`] and [`ParIter::chunk_size`] for details.
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets the iteration order of the parallel computation.
    ///
    /// See [`IterationOrder`] and [`ParIter::iteration_order`] for details.
    fn iteration_order(self, order: IterationOrder) -> Self;

    /// Rather than the [`DefaultRunner`], uses the parallel runner `Q` which implements [`ParallelRunner`].
    ///
    /// See [`ParIter::with_runner`] for details.
    fn with_runner<Q: ParallelRunner>(self) -> impl ParIterOption<Q, Item = Self::Item>;

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

    fn collect_into<C>(self, output: C) -> Option<C>
    where
        Self::Item: Send,
        C: ParCollectInto<Self::Item>;

    fn collect<C>(self) -> Option<C>
    where
        Self::Item: Send,
        C: ParCollectInto<Self::Item>;

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync;

    fn all<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        Self: Sized,
        Self::Item: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        let violates = |x: &Self::Item| !predicate(x);
        self.find(violates).map(|x| x.is_none())
    }

    fn any<Predicate>(self, predicate: Predicate) -> Option<bool>
    where
        Self: Sized,
        Self::Item: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        self.find(predicate).map(|x| x.is_some())
    }

    fn count(self) -> Option<usize>
    where
        Self: Sized,
    {
        self.map(map_count)
            .reduce(reduce_sum)
            .map(|x| x.unwrap_or(0))
    }

    fn for_each<Operation>(self, operation: Operation) -> Option<()>
    where
        Self: Sized,
        Operation: Fn(Self::Item) + Sync,
    {
        let map = |x| operation(x);
        self.map(map).reduce(reduce_unit).map(|_| ())
    }

    fn max(self) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::max)
    }

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

    fn min(self) -> Option<Option<Self::Item>>
    where
        Self: Sized,
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::min)
    }

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

    fn first(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Send;

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
