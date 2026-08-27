#![allow(clippy::type_complexity)]

use crate::common_par_traits::ParResCommon;
use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, MappedOf, Xap};
use crate::infallible_use::xap_variants::IdUse;
use crate::result::ParResultIter;
use crate::result::par_core::ParResultCore;
use crate::result_use::ParUseResultIter;
use crate::runner::ParRunner;
use crate::sizes::SizePair;
use crate::use_var::{UseSlice, UseVec};
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParUseResult, Sum};
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Fallible parallel iterator over `Result` values.
///
/// `ParResult` represents pipelines where each element may fail with an error.
/// It is commonly created from [`Par`](crate::Par) with
/// [`into_fallible`](crate::Par::into_fallible).
///
/// Conceptually, this is similar to using the `?` operator in Rust:
/// both let you keep logic on the success path while failures short-circuit.
/// In `ParResult`, the success path works with plain `T` values (instead of
/// `Result<T, E>`), and the parallel computation stops when an error is
/// observed.
///
/// Related traits:
/// - [`Par`](crate::Par) for infallible pipelines,
/// - [`ParUseResult`](crate::ParUseResult) for the same fallibility model with worker-local state.
///
/// # Examples
///
/// Parse and validate records in parallel.
///
/// ```
/// use orx_parallel::*;
///
/// let records = ["3", "8", "21", "34"];
///
/// let validated: Result<Vec<usize>, _> = records
///     .into_par()
///     .map(|s| s.parse::<usize>())
///     .into_fallible()
///     .map(|x| x * 2)
///     .filter(|x| *x <= 70)
///     .collect();
///
/// assert_eq!(validated, Ok(vec![6, 16, 42, 68]));
///
/// let with_failure: Result<Vec<usize>, _> = ["3", "bad", "21", "34"]
///     .into_par()
///     .map(|s| s.parse::<usize>())
///     .into_fallible()
///     .map(|x| x * 2)
///     .collect();
///
/// assert!(with_failure.is_err());
/// ```
pub trait ParResult:
    Sized + ParResultCore + ParResCommon<CommonItem = Self::Item, CommonError = Self::Error>
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
    ///     .into_fallible();
    ///
    /// let par = par.runner(Runner::fixed());
    ///
    /// let out: Result<Vec<_>, _> = par.collect();
    /// assert_eq!(out, Ok(vec![1, 2, 3]));
    /// ```
    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParResult<
        Item = Self::Item,
        Error = Self::Error,
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
    ///     .into_fallible();
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner_with_diagnostics();
    ///
    /// let out: Result<Vec<_>, _> = par.collect();
    /// assert_eq!(out, Ok(vec![1, 2, 3]));
    /// ```
    fn runner_with_diagnostics(
        self,
    ) -> impl ParResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    >;

    /// Sets the maximum number of worker threads for this computation.
    ///
    /// This method configures the **computation layer** of the thread count decision.
    /// The actual number of threads used is determined by combining:
    ///
    /// 1. **Pool constraint** (from `pool()` method or default pool)
    ///    - Already includes `ORX_NUM_THREADS` environment variable constraint
    /// 2. **Computation constraint** (this method)
    ///    - Your per-computation thread preference
    /// 3. **Input size constraint**
    ///    - Cannot spawn more threads than input elements
    ///
    /// The actual thread count is the **minimum** of all these constraints.
    ///
    /// # Parameter Interpretation
    ///
    /// Integer values map as follows:
    /// - `0` => `NumThreads::Auto` (use all available threads, spawn only as needed)
    /// - `n > 0` => `NumThreads::Max(n)` (cap at `n` threads)
    ///
    /// # Thread Count Decision Logic
    ///
    /// ```text
    /// available = pool.max_num_threads()      // Pool maximum (includes env variable)
    ///
    /// requested = match num_threads {
    ///     0 | Auto => input_size.max(1),      // Limited by input size
    ///     Max(n) => min(input_size, n),       // Limited by input size and this param
    /// };
    ///
    /// actual_threads = min(requested, available)
    /// ```
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use orx_parallel::*;
    ///
    /// // Sequential execution
    /// let out: Result<Vec<_>, _> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .num_threads(1)
    ///     .collect();
    /// assert_eq!(out, Ok(vec![1, 2, 3]));
    ///
    /// // Cap at 4 threads
    /// let out: Result<Vec<_>, _> = (1..1001)
    ///     .into_par()
    ///     .map(Ok::<_, String>)
    ///     .into_fallible()
    ///     .num_threads(4)
    ///     .collect();
    /// ```
    ///
    /// # Interaction with Pool
    ///
    /// The actual thread count respects the thread pool's constraints:
    ///
    /// ```ignore
    /// use orx_parallel::*;
    ///
    /// // Pool provides 4 threads max
    /// let pool = Pool::once(4);
    ///
    /// // Request 6 threads, but pool only has 4
    /// let out: Result<Vec<_>, _> = (1..1001)
    ///     .into_par()
    ///     .map(Ok::<_, String>)
    ///     .into_fallible()
    ///     .pool(pool)
    ///     .num_threads(6)  // Request 6...
    ///     .collect();      // ...but only 4 are available
    /// ```
    ///
    /// # See Also
    ///
    /// - [`NumThreads`](crate::NumThreads) - Type for thread configuration
    /// - [`pool()`](crate::Par::pool) - Configure thread pool
    /// - [`Pool`](crate::Pool) - Factory for creating pools
    /// - [`thread_usage.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/thread_usage.md) - Complete threading guide
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets chunk size used when pulling items from the concurrent input.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, _> = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .chunk_size(2)
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 2, 3, 4]));
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
    ///     .into_par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .iteration_order(IterationOrder::Ordered)
    ///     .find(|x| x % 3421 == 0);
    /// assert_eq!(ordered, Ok(Some(3421)));
    /// ```
    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    /// Creates one mutable `Use` value per participating worker.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    /// use rand::prelude::*;
    /// use rand_chacha::ChaCha8Rng;
    ///
    /// let out: Result<Vec<_>, _> = ["0", "1", "2", "3", "4", "5", "6", "7"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_new(|thread_idx| ChaCha8Rng::seed_from_u64(10 + thread_idx as u64))
    ///     .map(|rng, x| x + rng.random_range(0..10))
    ///     .collect();
    ///
    /// assert_eq!(out.as_ref().map(Vec::len), Ok(8));
    /// ```
    fn use_new<U, F>(
        self,
        f: F,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = U,
        Xap1 = IdUse<Self::Xap1, U>,
        M = Self::M,
        Xap2 = IdUse<Self::Xap2, U>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        U: Send,
        F: Fn(usize) -> U + Sync,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseVec::new(f);
        ParUseResultIter::new(u, iter, x1, x2, exe, params)
    }

    /// Uses an externally-owned [`UseVec`](crate::UseVec) as worker-local state.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut sums = UseVec::new(|_| 0usize);
    ///
    /// let result = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_vec(&mut sums)
    ///     .for_each(|local, x| *local += x);
    ///
    /// assert_eq!(result, Ok(()));
    /// assert_eq!(sums.into_vec().into_iter().sum::<usize>(), 55);
    /// ```
    fn use_vec<U, F>(
        self,
        use_vec: &mut UseVec<U, F>,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = U,
        Xap1 = IdUse<Self::Xap1, U>,
        M = Self::M,
        Xap2 = IdUse<Self::Xap2, U>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        U: Send,
        F: Fn(usize) -> U + Sync,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        ParUseResultIter::new(use_vec, iter, x1, x2, exe, params)
    }

    /// Uses a caller-provided mutable slice as worker-local mutable state.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut sums = vec![0usize; 4];
    /// let result = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .use_slice(&mut sums)
    ///     .for_each(|local, x| *local += x);
    ///
    /// assert_eq!(result, Ok(()));
    /// assert_eq!(sums.into_iter().sum::<usize>(), 55);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the input produces at least one element but `slice` is empty.
    fn use_slice<'a, U>(
        self,
        slice: &'a mut [U],
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = U,
        Xap1 = IdUse<Self::Xap1, U>,
        M = Self::M,
        Xap2 = IdUse<Self::Xap2, U>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        U: Sync + 'a,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseSlice::new(slice);
        ParUseResultIter::new(u, iter, x1, x2, exe, params)
    }

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
    ///     .copied()
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 2, 3]));
    /// ```
    fn copied<'a, O>(
        self,
    ) -> impl ParResult<
        Item = O,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, FnCopied<'a, O>>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        Self: ParResult<Item = &'a O>,
        O: Copy + 'a,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParResultIter::new(iter, x1, x2.mapped(FnCopied::new()), exe, params)
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
    ///     .cloned()
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec!["a".to_string(), "b".to_string()]));
    /// ```
    fn cloned<'a, O>(
        self,
    ) -> impl ParResult<
        Item = O,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, FnCloned<'a, O>>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        Self: ParResult<Item = &'a O>,
        O: Clone + 'a,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParResultIter::new(iter, x1, x2.mapped(FnCloned::new()), exe, params)
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
    ///     .map(|x| 2 * x)
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![2, 4, 6]));
    /// ```
    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParResult<
        Item = Q,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    /// Runs `h` on each successful element and forwards it unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::{AtomicUsize, Ordering};
    /// use orx_parallel::*;
    ///
    /// let seen = AtomicUsize::new(0);
    /// let out: Result<Vec<_>, _> = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .inspect(|_| { seen.fetch_add(1, Ordering::Relaxed); })
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 2, 3, 4]));
    /// assert_eq!(seen.load(Ordering::Relaxed), 4);
    /// ```
    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = InsOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = Self::Size,
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
    /// let out: Result<Vec<_>, _> = ["1", "2", "3", "4", "5", "6"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .filter(|x| x % 2 == 1)
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 3, 5]));
    /// ```
    fn filter<H>(
        self,
        h: H,
    ) -> impl ParResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
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
    ///     .into_par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 5]));
    /// ```
    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParResult<
        Item = Q,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilMapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
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
    /// let out: Result<Vec<_>, _> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .flat_map(|x| [x, x + 10])
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 11, 2, 12, 3, 13]));
    /// ```
    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParResult<
        Item = V::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlatMapOf<Self::Xap2, V, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
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
    /// let out: Result<Vec<_>, &'static str> = nested
    ///     .into_par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .flatten()
    ///     .collect();
    ///
    /// assert_eq!(out, Ok(vec![1, 2, 3, 4]));
    /// ```
    fn flatten(
        self,
    ) -> impl ParResult<
        Item = <Self::Item as IntoIterator>::Item,
        Error = Self::Error,
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
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!(["1", "2", "3"].into_par().map(|s| s.parse::<usize>()).into_fallible().first(), Ok(Some(1)));
    /// assert_eq!(Vec::<&str>::new().into_par().map(|s| s.parse::<usize>()).into_fallible().first(), Ok(None));
    /// ```
    fn first(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        Self::Error: Send;

    /// Reduces successful items into one value using `f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = (1..6)
    ///     .into_par()
    ///     .map(Ok::<_, &'static str>)
    ///     .into_fallible()
    ///     .reduce(|a, b| a + b);
    /// assert_eq!(ok, Ok(Some(15)));
    ///
    /// let fail = ["1", "x", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>().map_err(|_| "parse"))
    ///     .into_fallible()
    ///     .reduce(|a, b| a + b);
    /// assert_eq!(fail, Err("parse"));
    /// ```
    fn reduce<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send,
        Self::Error: Send;

    /// Collects successful items into `dst`.
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
    ///     .collect_into(&mut dst);
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
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Result<Vec<_>, _> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
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
    ///     .all(|x| x > &0);
    /// assert_eq!(ok, Ok(true));
    /// ```
    fn all<F>(self, f: F) -> Result<bool, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.map(|x| f(&x))
            .find(|x| !*x)
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
    ///     .any(|x| x % 2 == 0);
    /// assert_eq!(ok, Ok(true));
    /// ```
    fn any<F>(self, f: F) -> Result<bool, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.map(|x| f(&x)).find(|x| *x).map(|x| x.is_some())
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
    /// let n = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .filter(|x| x % 3 == 0)
    ///     .count();
    /// assert_eq!(n, Ok(3));
    /// ```
    fn count(self) -> Result<usize, Self::Error>
    where
        Self::Item: Send,
        Self::Error: Send,
    {
        self.map(|_| 1).reduce(|a, b| a + b).map(|x| x.unwrap_or(0))
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
    /// let numbers = (1..101).map(|x| x.to_string()).collect::<Vec<_>>();
    /// let found = numbers
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .find(|x| x % 17 == 0);
    /// assert_eq!(found, Ok(Some(17)));
    /// ```
    fn find<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.filter(&f).first()
    }

    /// Folds successful elements into per-thread accumulators.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let partials = ["1", "2", "3", "4", "5"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .fold(|| 0usize, |acc, x| *acc += x)
    ///     .unwrap();
    ///
    /// assert_eq!(partials.iter().sum::<usize>(), 15);
    /// ```
    fn fold<B, I, F>(self, init: I, f: F) -> Result<Vec<B>, Self::Error>
    where
        B: Send,
        I: Fn() -> B + Sync,
        F: Fn(&mut B, Self::Item) + Copy + Send,
        Self::Error: Send,
    {
        let mut use_vec = UseVec::new(|_| init());
        let par_use = self.use_vec(&mut use_vec);
        let result = par_use.for_each(move |u: &mut B, x| f(u, x));
        result.map(|_| use_vec.into_vec())
    }

    /// Executes `f` for each successful element.
    ///
    /// Returns `Err(e)` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::{AtomicUsize, Ordering};
    /// use orx_parallel::*;
    ///
    /// let total = AtomicUsize::new(0);
    /// let ok = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
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
    {
        self.map(f).reduce(|_, _| {}).map(|_| ())
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
    ///     .max();
    /// assert_eq!(m, Ok(Some(4)));
    /// ```
    fn max(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Ord + Send,
        Self::Error: Send,
    {
        self.reduce(Ord::max)
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
    ///     .max_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Ok(Some(5)));
    /// ```
    fn max_by<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
        Self::Error: Send,
    {
        let reduce = |x, y| match f(&x, &y) {
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
    ///     .max_by_key(|x| x.abs());
    /// assert_eq!(x, Ok(Some(-10)));
    /// ```
    fn max_by_key<B, F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
        Self::Error: Send,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
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
    ///     .min();
    /// assert_eq!(m, Ok(Some(1)));
    /// ```
    fn min(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Ord + Send,
        Self::Error: Send,
    {
        self.reduce(Ord::min)
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
    ///     .min_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Ok(Some(-10)));
    /// ```
    fn min_by<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
        Self::Error: Send,
    {
        let reduce = |x, y| match f(&x, &y) {
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
    ///     .min_by_key(|x| x.abs());
    /// assert_eq!(x, Ok(Some(0)));
    /// ```
    fn min_by_key<B, F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
        Self::Error: Send,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
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
    /// let ok: Result<usize, _> = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>())
    ///     .into_fallible()
    ///     .sum();
    /// assert_eq!(ok, Ok(10));
    /// ```
    fn sum<S>(self) -> Result<S, Self::Error>
    where
        Self::Item: Sum<S>,
        S: Send,
        Self::Error: Send,
    {
        self.map(Self::Item::owned)
            .reduce(Self::Item::add)
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }
}
