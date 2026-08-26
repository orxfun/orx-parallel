#![allow(clippy::type_complexity)]

use crate::infallible::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf};
use crate::result::recursive::par_core::ParRecResultCore;
use crate::runner::ParRunner;
use crate::{ChunkSize, IterationOrder, NumThreads};
use crate::{ParCollectInto, Sum};
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Fallible recursive parallel iterator over `Result` values.
///
/// `ParRecResult` is the recursive counterpart of [`ParResult`](crate::ParResult): each
/// visited node may fail with an error, in which case the whole computation short-circuits.
/// It is created from [`ParRec`](crate::ParRec) with
/// [`into_fallible`](crate::ParRec::into_fallible). When a node fails, its children are
/// **not** discovered/visited.
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// let children: Vec<Vec<usize>> = vec![vec![1, 2], vec![3, 4], vec![5], vec![], vec![], vec![]];
///
/// let mut ok: Result<Vec<usize>, &'static str> = [0usize]
///     .into_par_rec(|node| children[*node].iter().copied())
///     .map(|x| if x <= 5 { Ok(x) } else { Err("too large") })
///     .into_fallible()
///     .map(|x| x * 2)
///     .collect();
/// ok.as_mut().unwrap().sort();
/// assert_eq!(ok, Ok(vec![0, 2, 4, 6, 8, 10]));
///
/// let fail: Result<Vec<usize>, &'static str> = [0usize]
///     .into_par_rec(|node| children[*node].iter().copied())
///     .map(|x| if x != 5 { Ok(x) } else { Err("five") })
///     .into_fallible()
///     .map(|x| x * 2)
///     .collect();
/// assert_eq!(fail, Err("five"));
/// ```
pub trait ParRecResult: Sized + ParRecResultCore {
    // configuration

    /// Replaces the current parallel runner with `runner`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let children: Vec<Vec<i32>> = vec![vec![1, 2], vec![], vec![]];
    ///
    /// let par = [0i32]
    ///     .into_par_rec(|node| children[*node as usize].iter().copied())
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible();
    ///
    /// let par = par.runner(Runner::fixed());
    ///
    /// let mut out: Result<Vec<_>, _> = par.collect();
    /// out.as_mut().unwrap().sort();
    /// assert_eq!(out, Ok(vec![0, 1, 2]));
    /// ```
    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParRecResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
    >;

    /// Wraps the current parallel runner with a diagnostics-enabled runner.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "std")]
    /// # fn main() {
    /// use orx_parallel::*;
    ///
    /// let par = [1i32]
    ///     .into_par_rec(|&x| (x < 100).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .num_threads(4);
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner_with_diagnostics();
    ///
    /// let sum: Result<i32, _> = par.sum();
    /// assert_eq!(sum, Ok(5050));
    /// # }
    /// ```
    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParRecResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
    >;

    /// Sets the maximum number of worker threads for this computation.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let sum: Result<i32, &'static str> = [1i32]
    ///     .into_par_rec(|&x| (x < 5).then_some(x + 1))
    ///     .map(Ok)
    ///     .into_fallible()
    ///     .num_threads(2)
    ///     .sum();
    /// assert_eq!(sum, Ok(15));
    /// ```
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets chunk size used when pulling items from the concurrent input.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut values: Result<Vec<_>, &'static str> = [0usize]
    ///     .into_par_rec(|&x| (x < 31).then_some(x + 1))
    ///     .map(Ok)
    ///     .into_fallible()
    ///     .chunk_size(8)
    ///     .map(|x| x + 1)
    ///     .collect();
    ///
    /// let values = values.as_mut().unwrap();
    /// values.sort();
    /// assert_eq!(values.len(), 32);
    /// assert_eq!(values[0], 1);
    /// assert_eq!(values[31], 32);
    /// ```
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets iteration order semantics for operations sensitive to ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ordered = [1i32]
    ///     .into_par_rec(|&x| (x < 9_999).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .iteration_order(IterationOrder::Ordered)
    ///     .find(|x| x % 3421 == 0);
    /// assert_eq!(ordered, Ok(Some(3421)));
    /// ```
    fn iteration_order(self, collect: IterationOrder) -> Self;

    // transformations

    /// Maps each successful element with closure `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, &'static str> = [1i32]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .map(Ok)
    ///     .into_fallible()
    ///     .map(|x| 2 * x)
    ///     .collect();
    /// assert_eq!(out, Ok(vec![2, 4, 6]));
    /// ```
    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParRecResult<
        Item = Q,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
    >
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    /// Runs `h` on each successful element and forwards the element unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::{AtomicUsize, Ordering};
    /// use orx_parallel::*;
    ///
    /// let seen = AtomicUsize::new(0);
    /// let out: Result<Vec<_>, &'static str> = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Ok)
    ///     .into_fallible()
    ///     .inspect(|_| {
    ///         seen.fetch_add(1, Ordering::Relaxed);
    ///     })
    ///     .collect();
    ///
    /// assert_eq!(out.map(|mut v| { v.sort(); v }), Ok(vec![1, 2, 3, 4]));
    /// assert_eq!(seen.load(Ordering::Relaxed), 4);
    /// ```
    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParRecResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = InsOf<Self::Xap2, H>,
        Input = Self::Input,
    >
    where
        H: Fn(&Self::Item) + Copy + Send;

    /// Keeps successful elements satisfying predicate `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, &'static str> = [1i32]
    ///     .into_par_rec(|&x| (x < 6).then_some(x + 1))
    ///     .map(Ok)
    ///     .into_fallible()
    ///     .filter(|x| x % 2 == 1)
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 3, 5]));
    /// ```
    fn filter<H>(
        self,
        h: H,
    ) -> impl ParRecResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilOf<Self::Xap2, H>,
        Input = Self::Input,
    >
    where
        H: Fn(&Self::Item) -> bool + Copy + Send;

    /// Maps and filters successful elements in a single pass.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, &'static str> = ["1", "x", "5"]
    ///     .into_par_rec(|_: &&str| None::<&str>)
    ///     .map(Ok)
    ///     .into_fallible()
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 5]));
    /// ```
    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParRecResult<
        Item = Q,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilMapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
    >
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send;

    /// Maps each successful element to an iterator and flattens one level.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, &'static str> = [1i32]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .map(Ok)
    ///     .into_fallible()
    ///     .flat_map(|x| [x, x + 10])
    ///     .collect();
    /// assert_eq!(out, Ok(vec![1, 11, 2, 12, 3, 13]));
    /// ```
    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParRecResult<
        Item = V::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlatMapOf<Self::Xap2, V, H>,
        Input = Self::Input,
    >
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send;

    /// Flattens one level of nested iterables on the success path.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let nested = vec![vec![1, 2], vec![3, 4]];
    /// let mut out: Result<Vec<_>, &'static str> = nested
    ///     .into_par_rec(|_: &Vec<i32>| None::<Vec<i32>>)
    ///     .map(Ok)
    ///     .into_fallible()
    ///     .flatten()
    ///     .collect();
    /// out.as_mut().unwrap().sort();
    ///
    /// assert_eq!(out, Ok(vec![1, 2, 3, 4]));
    /// ```
    fn flatten(
        self,
    ) -> impl ParRecResult<
        Item = <Self::Item as IntoIterator>::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlattenOf<Self::Xap2>,
        Input = Self::Input,
    >
    where
        Self::Item: IntoIterator;

    // compute

    /// Returns the first successful item, or `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let first = [1i32]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .first();
    /// assert_eq!(first, Ok(Some(1)));
    /// ```
    fn first(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone;

    /// Reduces successful items into one value using `f`, or `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let reduced = [1i32]
    ///     .into_par_rec(|&x| (x < 5).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .reduce(|a, b| a + b);
    /// assert_eq!(reduced, Ok(Some(15)));
    /// ```
    fn reduce<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone;

    /// Collects successful items into `dst`, or returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut dst = vec![10];
    /// let ok = [0i32]
    ///     .into_par_rec(|&x| (x < 2).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .collect_into(&mut dst);
    /// assert_eq!(ok, Ok(()));
    /// dst.sort();
    /// assert_eq!(dst, vec![0, 1, 2, 10]);
    /// ```
    fn collect_into<C>(self, dst: &mut C) -> Result<(), Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone;

    /// Collects successful items into a new collection, or returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, &'static str> = [1i32]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .map(Ok)
    ///     .into_fallible()
    ///     .map(|x| x * 2)
    ///     .collect();
    /// assert_eq!(out, Ok(vec![2, 4, 6]));
    /// ```
    fn collect<C>(self) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone;

    // compute - derived

    /// Returns `Ok(true)` if all successful items satisfy `f`; `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .all(|x| x > &0);
    /// assert_eq!(out, Ok(true));
    /// ```
    fn all<F>(self, f: F) -> Result<bool, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.map(|x| f(&x))
            .find(|x| !*x)
            .map(|x| x.map(|_| false).unwrap_or(true))
    }

    /// Returns `Ok(true)` if any successful item satisfies `f`; `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .any(|x| x % 2 == 0);
    /// assert_eq!(out, Ok(true));
    /// ```
    fn any<F>(self, f: F) -> Result<bool, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.map(|x| f(&x)).find(|x| *x).map(|x| x.is_some())
    }

    /// Counts successful elements, or returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let n = [1i32]
    ///     .into_par_rec(|&x| (x < 10).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .filter(|x| x % 3 == 0)
    ///     .count();
    /// assert_eq!(n, Ok(3));
    /// ```
    fn count(self) -> Result<usize, Self::Error>
    where
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.map(|_| 1).reduce(|a, b| a + b).map(|x| x.unwrap_or(0))
    }

    /// Finds first successful item satisfying `f`, or `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let found = [1i32]
    ///     .into_par_rec(|&x| (x < 100).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .find(|x| x % 17 == 0);
    /// assert_eq!(found, Ok(Some(17)));
    /// ```
    fn find<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.filter(&f).first()
    }

    /// Folds successful elements into per-thread accumulators, or returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let partials = [1usize]
    ///     .into_par_rec(|&x| (x < 5).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .fold(|| 0usize, |acc, x| *acc += x);
    /// assert_eq!(partials.as_ref().map(|v| v.iter().sum::<usize>()), Ok(15));
    /// ```
    fn fold<B, I, F>(self, init: I, f: F) -> Result<Vec<B>, Self::Error>
    where
        B: Send + Sync,
        I: Fn() -> B + Sync,
        F: Fn(&mut B, Self::Item) + Copy + Send + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone;

    /// Executes `f` for each successful element, or returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::{AtomicUsize, Ordering};
    /// use orx_parallel::*;
    ///
    /// let total = AtomicUsize::new(0);
    /// let ok = [1usize]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .for_each(|x| {
    ///         total.fetch_add(x, Ordering::Relaxed);
    ///     });
    ///
    /// assert_eq!(ok, Ok(()));
    /// assert_eq!(total.load(Ordering::Relaxed), 10);
    /// ```
    fn for_each<F>(self, f: F) -> Result<(), Self::Error>
    where
        F: Fn(Self::Item) + Send + Copy,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.map(f).reduce(|_, _| {}).map(|_| ())
    }

    /// Returns maximum successful element, or `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let max = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .max();
    /// assert_eq!(max, Ok(Some(4)));
    /// ```
    fn max(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Ord + Send,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.reduce(Ord::max)
    }

    /// Returns successful element considered maximum by comparator `f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .max_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Ok(Some(5)));
    /// ```
    fn max_by<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let reduce = |x, y| match f(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns successful element with maximum key value.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .max_by_key(|x| x.abs());
    /// assert_eq!(x, Ok(Some(-10)));
    /// ```
    fn max_by_key<B, F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns minimum successful element, or `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let min = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .min();
    /// assert_eq!(min, Ok(Some(1)));
    /// ```
    fn min(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Ord + Send,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.reduce(Ord::min)
    }

    /// Returns successful element considered minimum by comparator `f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .min_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Ok(Some(-10)));
    /// ```
    fn min_by<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let reduce = |x, y| match f(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Returns successful element with minimum key value.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .min_by_key(|x| x.abs());
    /// assert_eq!(x, Ok(Some(0)));
    /// ```
    fn min_by_key<B, F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Sums successful elements using the [`Sum`] implementation, or `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let sum: Result<usize, &'static str> = [1usize]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Ok)
    ///     .into_fallible()
    ///     .sum();
    /// assert_eq!(sum, Ok(10));
    /// ```
    fn sum<S>(self) -> Result<S, Self::Error>
    where
        Self::Item: Sum<S>,
        S: Send,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.map(Self::Item::owned)
            .reduce(Self::Item::add)
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }
}
