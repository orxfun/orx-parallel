use crate::computations::{map_count, reduce_sum, reduce_unit};
use crate::{ChunkSize, IterationOrder, NumThreads, Sum};
use crate::{
    DefaultRunner, ParCollectInto, ParIter, ParallelRunner,
    generic_values::fallible_iterators::ResultOfIter,
};
use core::cmp::Ordering;

#[test]
fn abc() {
    use crate::*;

    // all succeeds
    let words: Vec<Result<&str, char>> = vec![Ok("alpha"), Ok("beta"), Ok("gamma")];

    let all_chars: Result<Vec<_>, _> = words
        .into_par()
        .into_fallible_result()
        .flat_map(|s| s.chars()) // chars() returns an iterator
        .collect();

    let merged: Result<String, _> = all_chars.map(|chars| chars.iter().collect());
    assert_eq!(merged, Ok("alphabetagamma".to_string()));

    // at least one fails
    let words: Vec<Result<&str, char>> = vec![Ok("alpha"), Ok("beta"), Err('x'), Ok("gamma")];

    let all_chars: Result<Vec<_>, _> = words
        .into_par()
        .into_fallible_result()
        .flat_map(|s| s.chars()) // chars() returns an iterator
        .collect();

    let merged: Result<String, _> = all_chars.map(|chars| chars.iter().collect());
    assert_eq!(merged, Err('x'));

    // all succeeds
    let a: Vec<Result<u32, char>> = vec![Ok(1), Ok(2), Ok(3)];
    let iter = a.into_par().into_fallible_result().map(|x| 2 * x);

    let b: Result<Vec<_>, _> = iter.collect();
    assert_eq!(b, Ok(vec![2, 4, 6]));

    // at least one fails
    let a = vec![Ok(1), Err('x'), Ok(3)];
    let iter = a.into_par().into_fallible_result().map(|x| 2 * x);

    let b: Result<Vec<_>, _> = iter.collect();
    assert_eq!(b, Err('x'));
}

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
pub trait ParIterResult<R = DefaultRunner>
where
    R: ParallelRunner,
{
    /// Type of the Ok element, to be received as the Ok variant iff the entire computation succeeds.
    type Ok;

    /// Type of the Err element, to be received if any of the computations fails.
    type Err: Send;

    /// Element type of the regular parallel iterator this fallible iterator can be converted to, simply `Result<Self::Ok, Self::Err>`.
    type RegularItem: IntoResult<Self::Ok, Self::Err>;

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

    /// Rather than the [`DefaultRunner`], uses the parallel runner `Q` which implements [`ParallelRunner`].
    ///
    /// See [`ParIter::with_runner`] for details.
    fn with_runner<Q: ParallelRunner>(
        self,
    ) -> impl ParIterResult<Q, Ok = Self::Ok, Err = Self::Err>;

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
    fn map<Out, Map>(self, map: Map) -> impl ParIterResult<R, Ok = Out, Err = Self::Err>
    where
        Self: Sized,
        Map: Fn(Self::Ok) -> Out + Sync + Clone,
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
    fn filter<Filter>(self, filter: Filter) -> impl ParIterResult<R, Ok = Self::Ok, Err = Self::Err>
    where
        Self: Sized,
        Filter: Fn(&Self::Ok) -> bool + Sync + Clone,
        Self::Ok: Send,
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
    ) -> impl ParIterResult<R, Ok = IOut::Item, Err = Self::Err>
    where
        Self: Sized,
        IOut: IntoIterator,
        IOut::Item: Send,
        FlatMap: Fn(Self::Ok) -> IOut + Sync + Clone,
    {
        let par = self.into_regular_par();
        let map = par.flat_map(move |x| match x.into_result() {
            Ok(x) => ResultOfIter::ok(flat_map(x).into_iter()),
            Err(e) => ResultOfIter::err(e),
        });
        map.into_fallible_result()
    }

    fn filter_map<Out, FilterMap>(
        self,
        filter_map: FilterMap,
    ) -> impl ParIterResult<R, Ok = Out, Err = Self::Err>
    where
        Self: Sized,
        FilterMap: Fn(Self::Ok) -> Option<Out> + Sync + Clone,
        Out: Send,
    {
        let par = self.into_regular_par();
        let filter_map = par.filter_map(move |x| match x.into_result() {
            Ok(x) => filter_map(x).map(|x| Ok(x)),
            Err(e) => Some(Err(e)),
        });
        filter_map.into_fallible_result()
    }

    fn inspect<Operation>(
        self,
        operation: Operation,
    ) -> impl ParIterResult<R, Ok = Self::Ok, Err = Self::Err>
    where
        Self: Sized,
        Operation: Fn(&Self::Ok) + Sync + Clone,
        Self::Ok: Send,
    {
        let map = move |x| {
            operation(&x);
            x
        };
        self.map(map)
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Err>
    where
        C: ParCollectInto<Self::Ok>;

    fn collect<C>(self) -> Result<C, Self::Err>
    where
        Self: Sized,
        C: ParCollectInto<Self::Ok>,
    {
        let output = C::empty(self.con_iter_len());
        self.collect_into(output)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Ok>, Self::Err>
    where
        Self::Ok: Send,
        Reduce: Fn(Self::Ok, Self::Ok) -> Self::Ok + Sync;

    fn all<Predicate>(self, predicate: Predicate) -> Result<bool, Self::Err>
    where
        Self: Sized,
        Self::Ok: Send,
        Predicate: Fn(&Self::Ok) -> bool + Sync,
    {
        let violates = |x: &Self::Ok| !predicate(x);
        self.find(violates).map(|x| x.is_none())
    }

    fn any<Predicate>(self, predicate: Predicate) -> Result<bool, Self::Err>
    where
        Self: Sized,
        Self::Ok: Send,
        Predicate: Fn(&Self::Ok) -> bool + Sync,
    {
        self.find(predicate).map(|x| x.is_some())
    }

    fn count(self) -> Result<usize, Self::Err>
    where
        Self: Sized,
    {
        self.map(map_count)
            .reduce(reduce_sum)
            .map(|x| x.unwrap_or(0))
    }

    fn for_each<Operation>(self, operation: Operation) -> Result<(), Self::Err>
    where
        Self: Sized,
        Operation: Fn(Self::Ok) + Sync,
    {
        let map = |x| operation(x);
        self.map(map).reduce(reduce_unit).map(|_| ())
    }

    fn max(self) -> Result<Option<Self::Ok>, Self::Err>
    where
        Self: Sized,
        Self::Ok: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    fn max_by<Compare>(self, compare: Compare) -> Result<Option<Self::Ok>, Self::Err>
    where
        Self: Sized,
        Self::Ok: Send,
        Compare: Fn(&Self::Ok, &Self::Ok) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Result<Option<Self::Ok>, Self::Err>
    where
        Self: Sized,
        Self::Ok: Send,
        Key: Ord,
        GetKey: Fn(&Self::Ok) -> Key + Sync,
    {
        let reduce = |x, y| match key(&x).cmp(&key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn min(self) -> Result<Option<Self::Ok>, Self::Err>
    where
        Self: Sized,
        Self::Ok: Ord + Send,
    {
        self.reduce(Ord::min)
    }

    fn min_by<Compare>(self, compare: Compare) -> Result<Option<Self::Ok>, Self::Err>
    where
        Self: Sized,
        Self::Ok: Send,
        Compare: Fn(&Self::Ok, &Self::Ok) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Result<Option<Self::Ok>, Self::Err>
    where
        Self: Sized,
        Self::Ok: Send,
        Key: Ord,
        GetKey: Fn(&Self::Ok) -> Key + Sync,
    {
        let reduce = |x, y| match get_key(&x).cmp(&get_key(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn sum<Out>(self) -> Result<Out, Self::Err>
    where
        Self: Sized,
        Self::Ok: Sum<Out>,
        Out: Send,
    {
        self.map(Self::Ok::map)
            .reduce(Self::Ok::reduce)
            .map(|x| x.unwrap_or(Self::Ok::zero()))
    }

    // early exit

    fn first(self) -> Result<Option<Self::Ok>, Self::Err>
    where
        Self::Ok: Send;

    fn find<Predicate>(self, predicate: Predicate) -> Result<Option<Self::Ok>, Self::Err>
    where
        Self: Sized,
        Self::Ok: Send,
        Predicate: Fn(&Self::Ok) -> bool + Sync,
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
