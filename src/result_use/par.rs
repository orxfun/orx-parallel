#![allow(clippy::type_complexity)]

use crate::common_par_traits::ParResCommon;
use crate::infallible_use::fun::{UFnCloned, UFnCopied};
use crate::infallible_use::{
    FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, MappedOf, XapUse,
};
use crate::pool::ParThreadPool;
use crate::result_use::{ParUseResultCore, ParUseResultIter};
use crate::runner::ParRunner;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, Sum};
use core::cmp::Ordering;

/// Fallible parallel iterator with worker-local mutable state.
///
/// `ParUseResult` combines:
/// - fallible processing (`Result`-based short-circuiting), and
/// - mutable worker-local state (`&mut Use` passed to closures).
///
/// You can enter this mode from [`ParResult`](crate::ParResult) via
/// [`use_new`](crate::ParResult::use_new),
/// [`use_vec`](crate::ParResult::use_vec), or
/// [`use_slice`](crate::ParResult::use_slice).
///
/// Similar to using `?`, this trait keeps pipeline logic focused on successful
/// values while computation short-circuits to `Err(e)` when any element fails.
///
/// Related traits:
/// - [`ParResult`](crate::ParResult) for `Result`-fallible pipelines without worker-local state,
/// - [`ParUse`](crate::ParUse) for worker-local state in infallible pipelines.
///
/// # Examples
///
/// Reusing a per-worker buffer:
///
/// ```
/// use core::fmt::Write;
/// use orx_parallel::*;
///
/// let out: Result<Vec<usize>, _> = ["0", "1", "2", "3", "4", "5", "6", "7"]
///     .into_par()
///     .map(|s| s.parse::<usize>())
///     .into_fallible()
///     .use_new(|_| String::with_capacity(32))
///     .map(|buffer, x| {
///         buffer.clear();
///         write!(buffer, "{x}").unwrap();
///         buffer.parse::<usize>().unwrap()
///     })
///     .collect();
///
/// assert_eq!(out, Ok((0..8).collect::<Vec<_>>()));
/// ```
///
/// Using RNG as mutable worker-local state:
///
/// ```
/// use orx_parallel::*;
/// use rand::prelude::*;
/// use rand_chacha::ChaCha8Rng;
///
/// let out: Result<Vec<usize>, _> = ["0", "1", "2", "3", "4", "5", "6", "7"]
///     .into_par()
///     .map(|s| s.parse::<usize>())
///     .into_fallible()
///     .use_new(|thread_idx| ChaCha8Rng::seed_from_u64(100 + thread_idx as u64))
///     .map(|rng, x| x + rng.random_range(0..10))
///     .collect();
///
/// assert_eq!(out.as_ref().map(Vec::len), Ok(8));
/// ```
pub trait ParUseResult:
    Sized + ParUseResultCore + ParResCommon<CommonItem = Self::Item, CommonError = Self::Error>
{
    // configuration

    /// Replaces the current parallel runner with `runner`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let par = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ());
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner(Runner::fixed(Pool::once(4)));
    ///
    /// let out: Result<Vec<_>, _> = par.collect();
    /// assert_eq!(out, Ok(vec![1, 2, 3]));
    /// ```
    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    >;

    #[cfg(feature = "std")]
    /// Wraps the current runner with diagnostics-enabled execution.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let par = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ());
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner_with_diagnostics();
    ///
    /// let out: Result<Vec<_>, _> = par.collect();
    /// assert_eq!(out, Ok(vec![1, 2, 3]));
    /// ```
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    >;

    /// Replaces the pool used by the current runner.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// #[cfg(feature = "std")]
    /// {
    ///     let out: Result<Vec<_>, _> = ["1", "2", "3"]
    ///         .into_par()
    ///         .map(|s| s.parse::<usize>())
    ///         .into_fallible()
    ///         .use_new(|_| ())
    ///         .pool(Pool::once(4))
    ///         .collect();
    ///
    ///     assert_eq!(out, Ok(vec![1, 2, 3]));
    /// }
    /// ```
    fn pool<P: ParThreadPool>(
        self,
        pool: P,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    >;

    /// Sets the maximum number of worker threads for this computation.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, _> = ["1", "2", "3", "4", "5"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .num_threads(1)
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 2, 3, 4, 5]));
    /// ```
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets chunk size used when pulling items from the concurrent input.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, _> = ["0", "1", "2", "3", "4", "5", "6", "7"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .chunk_size(2)
    ///     .collect();
    ///
    /// assert_eq!(out, Ok((0..8).collect::<Vec<_>>()));
    /// ```
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets iteration-order semantics for order-sensitive operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ordered = (1..10_000)
    ///     .map(|x| x.to_string())
    ///     .collect::<Vec<_>>()
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .iteration_order(IterationOrder::Ordered)
    ///     .find(|_, x| x % 3421 == 0);
    ///
    /// assert_eq!(ordered, Ok(Some(3421)));
    /// ```
    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    /// Copies elements of a reference iterator on the success path.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec![1, 2, 3];
    /// let out: Result<Vec<_>, &'static str> = data
    ///     .par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .copied()
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 2, 3]));
    /// ```
    fn copied<'a, O>(
        self,
    ) -> impl ParUseResult<
        Item = O,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, UFnCopied<'a, Self::Use, O>>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        Self: ParUseResult<Item = &'a O>,
        O: Copy + 'a,
        Self::Use: 'a,
    {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseResultIter::new(u, iter, x1, x2.mapped(UFnCopied::new()), exe, params)
    }

    /// Clones elements of a reference iterator on the success path.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec!["a".to_string(), "b".to_string()];
    /// let out: Result<Vec<_>, &'static str> = data
    ///     .par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .cloned()
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec!["a".to_string(), "b".to_string()]));
    /// ```
    fn cloned<'a, O>(
        self,
    ) -> impl ParUseResult<
        Item = O,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, UFnCloned<'a, Self::Use, O>>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        Self: ParUseResult<Item = &'a O>,
        O: Clone + 'a,
        Self::Use: 'a,
    {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseResultIter::new(u, iter, x1, x2.mapped(UFnCloned::new()), exe, params)
    }

    // transformations

    /// Maps each successful element with closure `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, _> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .map(|_, x| 2 * x)
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![2, 4, 6]));
    /// ```
    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Item = Q,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        H: Fn(&mut Self::Use, Self::Item) -> Q + Copy + Send;

    /// Runs `h` on each successful element and forwards the item unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut use_vec = UseVec::new(|_| 0usize);
    /// let out: Result<Vec<_>, _> = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_vec(&mut use_vec)
    ///     .inspect(|count, _| *count += 1)
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 2, 3, 4]));
    /// assert_eq!(use_vec.into_vec().into_iter().sum::<usize>(), 4);
    /// ```
    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = InsOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        H: Fn(&mut Self::Use, &Self::Item) + Copy + Send;

    /// Keeps successful elements satisfying predicate `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, _> = ["1", "2", "3", "4", "5", "6"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .filter(|_, x| x % 2 == 1)
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 3, 5]));
    /// ```
    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
    >
    where
        H: Fn(&mut Self::Use, &Self::Item) -> bool + Copy + Send;

    /// Maps and filters successful elements in one pass.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, _> = ["1", "x", "5"]
    ///     .into_par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .filter_map(|_, s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 5]));
    /// ```
    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Item = Q,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilMapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
    >
    where
        H: Fn(&mut Self::Use, Self::Item) -> Option<Q> + Copy + Send;

    /// Maps each successful element to an iterator and flattens one level.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, _> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .flat_map(|_, x| [x, x + 10])
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 11, 2, 12, 3, 13]));
    /// ```
    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Item = V::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlatMapOf<Self::Xap2, V, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
    >
    where
        V: IntoIterator,
        H: Fn(&mut Self::Use, Self::Item) -> V + Copy + Send;

    /// Flattens one level of nested iterables on the success path.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let nested = vec![vec![1, 2], vec![3, 4]];
    /// let out: Result<Vec<_>, &'static str> = nested
    ///     .into_par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .flatten()
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 2, 3, 4]));
    /// ```
    fn flatten(
        self,
    ) -> impl ParUseResult<
        Item = <Self::Item as IntoIterator>::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlattenOf<Self::Xap2>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
    >
    where
        Self::Item: IntoIterator;

    // compute

    /// Returns the first successful item according to iteration order.
    ///
    /// Returns:
    /// - `Err(e)` if computation short-circuits due to a failure
    /// - `Ok(None)` if no successful element exists
    /// - `Ok(Some(x))` for the first successful element
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .first();
    /// assert_eq!(ok, Ok(Some(1)));
    ///
    /// let fail = ["bad", "1", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .first();
    /// assert!(fail.is_err());
    /// ```
    fn first(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        Self::Error: Send;

    /// Reduces successful items into one value using `f`.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = ["1", "2", "3", "4", "5"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .reduce(|_, a, b| a + b);
    /// assert_eq!(ok, Ok(Some(15)));
    ///
    /// let fail = ["1", "x", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>().map_err(|_| "parse"))
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .reduce(|_, a, b| a + b);
    /// assert_eq!(fail, Err("parse"));
    /// ```
    fn reduce<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        F: Fn(&mut Self::Use, Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send,
        Self::Error: Send;

    /// Collects successful items into `dst`.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut dst = vec![10usize];
    /// let ok = ["0", "1", "2"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .collect_into(&mut dst);
    ///
    /// assert_eq!(ok, Ok(()));
    /// assert_eq!(dst, vec![10, 0, 1, 2]);
    /// ```
    fn collect_into<C>(self, dst: &mut C) -> Result<(), Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
        Self::Error: Send;

    /// Collects successful items into a new collection.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, _> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 2, 3]));
    /// ```
    fn collect<C>(self) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
        Self::Error: Send;

    // compute - derived

    /// Returns `Ok(true)` if all successful items satisfy `f`.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .all(|_, x| x > &0);
    /// assert_eq!(ok, Ok(true));
    /// ```
    fn all<F>(self, f: F) -> Result<bool, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.map(|u, x| f(u, &x))
            .find(|_, x| !*x)
            .map(|x| x.map(|_| false).unwrap_or(true))
    }

    /// Returns `Ok(true)` if any successful item satisfies `f`.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .any(|_, x| x % 2 == 0);
    /// assert_eq!(ok, Ok(true));
    /// ```
    fn any<F>(self, f: F) -> Result<bool, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.map(|u, x| f(u, &x))
            .find(|_, x| *x)
            .map(|x| x.is_some())
    }

    /// Counts successful elements.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .filter(|_, x| x % 3 == 0)
    ///     .count();
    ///
    /// assert_eq!(ok, Ok(3));
    /// ```
    fn count(self) -> Result<usize, Self::Error>
    where
        Self::Item: Send,
        Self::Error: Send,
    {
        self.map(|_, _| 1)
            .reduce(|_, a, b| a + b)
            .map(|x| x.unwrap_or(0))
    }

    /// Finds first (ordered) or any (arbitrary) successful item satisfying `f`.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let found = (1..101)
    ///     .map(|x| x.to_string())
    ///     .collect::<Vec<_>>()
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .find(|_, x| x % 17 == 0);
    ///
    /// assert_eq!(found, Ok(Some(17)));
    /// ```
    fn find<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.filter(&f).first()
    }

    /// Executes `f` for each successful element.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut sums = UseVec::new(|_| 0usize);
    /// let result = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_vec(&mut sums)
    ///     .for_each(|local, x| *local += x);
    ///
    /// assert_eq!(result, Ok(()));
    /// assert_eq!(sums.into_vec().into_iter().sum::<usize>(), 10);
    /// ```
    fn for_each<F>(self, f: F) -> Result<(), Self::Error>
    where
        F: Fn(&mut Self::Use, Self::Item) + Send + Copy,
        Self::Error: Send,
    {
        self.map(f).reduce(|_, _, _| {}).map(|_| ())
    }

    /// Returns maximum successful element.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let m = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .max();
    ///
    /// assert_eq!(m, Ok(Some(4)));
    /// ```
    fn max(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Ord + Send,
        Self::Error: Send,
    {
        self.reduce(|_, a, b| Ord::max(a, b))
    }

    /// Returns successful element considered maximum by comparator `f`.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .max_by(|_, a, b| a.cmp(b));
    ///
    /// assert_eq!(x, Ok(Some(5)));
    /// ```
    fn max_by<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item, &Self::Item) -> Ordering + Sync,
        Self::Error: Send,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns successful element with maximum key value.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .max_by_key(|_, x| x.abs());
    ///
    /// assert_eq!(x, Ok(Some(-10)));
    /// ```
    fn max_by_key<B, F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&mut Self::Use, &Self::Item) -> B + Sync,
        Self::Error: Send,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x).cmp(&f(u, &y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns minimum successful element.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let m = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .min();
    ///
    /// assert_eq!(m, Ok(Some(1)));
    /// ```
    fn min(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Ord + Send,
        Self::Error: Send,
    {
        self.reduce(|_, a, b| Ord::min(a, b))
    }

    /// Returns successful element considered minimum by comparator `f`.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .min_by(|_, a, b| a.cmp(b));
    ///
    /// assert_eq!(x, Ok(Some(-10)));
    /// ```
    fn min_by<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item, &Self::Item) -> Ordering + Sync,
        Self::Error: Send,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Returns successful element with minimum key value.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .min_by_key(|_, x| x.abs());
    ///
    /// assert_eq!(x, Ok(Some(0)));
    /// ```
    fn min_by_key<B, F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&mut Self::Use, &Self::Item) -> B + Sync,
        Self::Error: Send,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x).cmp(&f(u, &y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Sums successful elements using [`Sum`] implementation.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let s: Result<usize, _> = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|_| ())
    ///     .sum();
    ///
    /// assert_eq!(s, Ok(10));
    /// ```
    fn sum<S>(self) -> Result<S, Self::Error>
    where
        Self::Item: Sum<S>,
        S: Send,
        Self::Error: Send,
    {
        self.map(|_, x| Self::Item::owned(x))
            .reduce(|_, a, b| Self::Item::add(a, b))
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }
}
