#![allow(clippy::type_complexity)]

use crate::common_par_traits::ParOptCommon;
use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, MappedOf, Xap};
use crate::infallible_use::xap_variants::IdUse;
use crate::option::ParOptionIter;
use crate::pool::ParThreadPool;
use crate::runner::ParRunner;
use crate::sizes::SizePair;
use crate::use_var::{UseSlice, UseVec};
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParUseOption, Sum};
use crate::{option::ParOptionCore, option_use::ParUseOptionIter};
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Fallible parallel iterator over `Option` values.
///
/// `ParOption` represents pipelines where each element may fail as `None`.
/// It is commonly created from [`Par`](crate::Par) with
/// [`into_optional`](crate::Par::into_optional).
///
/// Conceptually, this is similar to using the `?` operator in Rust:
/// both let you write logic on the success path while failures short-circuit.
/// In `ParOption`, the success path works with plain `T` values (instead of
/// `Option<T>`), and the parallel computation stops immediately when any
/// element evaluates to `None`.
///
/// Related traits:
/// - [`Par`](crate::Par) for infallible pipelines,
/// - [`ParUseOption`](crate::ParUseOption) for the same fallibility model with worker-local state.
///
/// # Examples
///
/// Parse and validate incoming records in parallel.
/// If any record is invalid, the pipeline short-circuits to `None`.
///
/// ```
/// use orx_parallel::*;
///
/// let records = ["3", "8", "21", "34"];
///
/// let validated: Option<Vec<usize>> = records
///     .into_par()
///     .map(|s| s.parse::<usize>().ok())
///     .into_optional()
///     .map(|x| x * 2)
///     .filter(|x| *x <= 70)
///     .collect();
///
/// assert_eq!(validated, Some(vec![6, 16, 42, 68]));
///
/// let with_failure: Option<Vec<usize>> = ["3", "bad", "21", "34"]
///     .into_par()
///     .map(|s| s.parse::<usize>().ok())
///     .into_optional()
///     .map(|x| x * 2)
///     .collect();
///
/// assert_eq!(with_failure, None);
/// ```
pub trait ParOption: Sized + ParOptionCore + ParOptCommon<CommonItem = Self::Item> {
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
    ///     .map(|s| s.parse::<usize>().ok())
    ///     .into_optional();
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner(Runner::fixed_chunk(Pool::once(4)));
    ///
    /// let out: Option<Vec<_>> = par.collect();
    /// assert_eq!(out, Some(vec![1, 2, 3]));
    /// ```
    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParOption<
        Item = Self::Item,
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
    ///     .map(|s| s.parse::<usize>().ok())
    ///     .into_optional();
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner_with_diagnostics();
    ///
    /// let out: Option<Vec<_>> = par.collect();
    /// assert_eq!(out, Some(vec![1, 2, 3]));
    /// ```
    fn runner_with_diagnostics(
        self,
    ) -> impl ParOption<
        Item = Self::Item,
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
    ///     let out: Option<Vec<_>> = ["1", "2", "3"]
    ///         .into_par()
    ///         .map(|s| s.parse::<usize>().ok())
    ///         .into_optional()
    ///         .pool(Pool::once(4))
    ///         .collect();
    ///     assert_eq!(out, Some(vec![1, 2, 3]));
    /// }
    ///
    /// #[cfg(feature = "rayon-core")]
    /// {
    ///     let out: Option<Vec<_>> = ["1", "2", "3"]
    ///         .into_par()
    ///         .map(|s| s.parse::<usize>().ok())
    ///         .into_optional()
    ///         .pool(Pool::rayon(4).unwrap())
    ///         .collect();
    ///     assert_eq!(out, Some(vec![1, 2, 3]));
    /// }
    /// ```
    fn pool<P: ParThreadPool>(
        self,
        pool: P,
    ) -> impl ParOption<
        Item = Self::Item,
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
    ///    - Already includes `ORX_PARALLEL_MAX_NUM_THREADS` environment variable constraint
    /// 2. **Computation constraint** (this method)
    ///    - Your per-computation thread preference
    /// 3. **Input size constraint**
    ///    - Cannot spawn more threads than input elements
    ///
    /// The actual thread count is the **minimum** of all these constraints.
    ///
    /// # Parameter Interpretation
    ///
    /// - `0` → `NumThreads::Auto` (use all available threads, spawn only as needed)
    /// - `n > 0` → `NumThreads::Max(n)` (cap at `n` threads)
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
    /// use std::num::NonZeroUsize;
    ///
    /// // Auto: uses all available threads (respects ORX_PARALLEL_MAX_NUM_THREADS)
    /// let out: Option<Vec<_>> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>().ok())
    ///     .into_optional()
    ///     .num_threads(NumThreads::Auto)
    ///     .collect();
    /// assert_eq!(out, Some(vec![1, 2, 3]));
    ///
    /// // Sequential execution (1 thread, no parallelism)
    /// let out: Option<Vec<_>> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>().ok())
    ///     .into_optional()
    ///     .num_threads(1)  // Sequential
    ///     .collect();
    /// assert_eq!(out, Some(vec![1, 2, 3]));
    ///
    /// // Cap at 4 threads
    /// let out: Option<Vec<_>> = (0..1000)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .num_threads(4)  // Use at most 4 threads
    ///     .collect();
    /// assert_eq!(out.as_ref().map(|v| v.len()), Some(1000));
    ///
    /// // With environment constraint: ORX_PARALLEL_MAX_NUM_THREADS=2
    /// let out: Option<Vec<_>> = (0..1000)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .num_threads(4)  // Request 4, but env limits to 2
    ///     .collect();      // Result: 2 threads used
    /// ```
    ///
    /// # Interaction with Pool
    ///
    /// The actual thread count also respects the thread pool's constraints:
    ///
    /// ```ignore
    /// use orx_parallel::*;
    ///
    /// // Pool provides 4 threads max
    /// let pool = Pool::once(4);
    ///
    /// // Computation requests 6 threads, but pool only has 4
    /// let result = (0..1000)
    ///     .into_par()
    ///     .map(|x| x * 2)
    ///     .pool(pool)
    ///     .num_threads(6)  // Request 6...
    ///     .collect();      // ...but only 4 are available
    /// ```
    ///
    /// # Checking Sequential Execution
    ///
    /// ```ignore
    /// let nt = NumThreads::Max(std::num::NonZeroUsize::new(1).unwrap());
    /// assert!(nt.is_sequential());  // true
    /// ```
    ///
    /// # See Also
    ///
    /// - [`NumThreads`](crate::NumThreads) - Type for thread configuration
    /// - [`pool()`](crate::Par::pool) - Configure thread pool
    /// - [`Pool`](crate::Pool) - Factory for creating pools
    /// - [`threading_model.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/threading_model.md) - Complete threading guide
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets chunk size used when pulling items from the concurrent input.
    ///
    /// Integer values map as follows:
    /// - `0` => automatic (default)
    /// - `n > 0` => exact chunk size `n`
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = ["1", "2", "3", "4"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>().ok())
    ///     .into_optional()
    ///     .chunk_size(2)
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 2, 3, 4]));
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
    ///     .map(|x| Some(x))
    ///     .into_optional()
    ///     .iteration_order(IterationOrder::Ordered)
    ///     .find(|x| x % 3421 == 0);
    /// assert_eq!(ordered, Some(Some(3421)));
    ///
    /// let any = (1..10_000)
    ///     .into_par()
    ///     .map(|x| Some(x))
    ///     .into_optional()
    ///     .iteration_order(IterationOrder::Arbitrary)
    ///     .find(|x| x % 3421 == 0)
    ///     .unwrap()
    ///     .unwrap();
    /// assert!([3421, 6842].contains(&any));
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
    /// let out: Option<Vec<_>> = (0..8usize)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|thread_idx| ChaCha8Rng::seed_from_u64(10 + thread_idx as u64))
    ///     .map(|rng, x| x + rng.random_range(0..10))
    ///     .collect();
    ///
    /// assert_eq!(out.as_ref().map(Vec::len), Some(8));
    /// ```
    fn use_new<U, F>(
        self,
        f: F,
    ) -> impl ParUseOption<
        Item = Self::Item,
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
        ParUseOptionIter::new(u, iter, x1, x2, exe, params)
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
    /// let result = (1..11)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_vec(&mut sums)
    ///     .for_each(|local, x| *local += x);
    ///
    /// assert_eq!(result, Some(()));
    /// assert_eq!(sums.into_vec().into_iter().sum::<usize>(), 55);
    /// ```
    fn use_vec<U, F>(
        self,
        use_vec: &mut UseVec<U, F>,
    ) -> impl ParUseOption<
        Item = Self::Item,
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
        ParUseOptionIter::new(use_vec, iter, x1, x2, exe, params)
    }

    /// Uses a caller-provided mutable slice as worker-local mutable state.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut sums = vec![0usize; 4];
    /// let result = (1..11)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_slice(&mut sums)
    ///     .for_each(|local, x| *local += x);
    ///
    /// assert_eq!(result, Some(()));
    /// assert_eq!(sums.into_iter().sum::<usize>(), 55);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the input produces at least one element but `slice` is empty.
    fn use_slice<'a, U>(
        self,
        slice: &'a mut [U],
    ) -> impl ParUseOption<
        Item = Self::Item,
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
        ParUseOptionIter::new(u, iter, x1, x2, exe, params)
    }

    /// Copies elements of a reference iterator.
    ///
    /// Equivalent to `.map(|x| *x)` on the success path.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec![1, 2, 3];
    /// let copied: Option<Vec<_>> = data.par().map(Some).into_optional().copied().collect();
    ///
    /// assert_eq!(copied, Some(vec![1, 2, 3]));
    /// ```
    fn copied<'a, O>(
        self,
    ) -> impl ParOption<
        Item = O,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, FnCopied<'a, O>>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        Self: ParOption<Item = &'a O>,
        O: Copy + 'a,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParOptionIter::new(iter, x1, x2.mapped(FnCopied::new()), exe, params)
    }

    /// Clones elements of a reference iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec!["a".to_string(), "b".to_string()];
    /// let cloned: Option<Vec<_>> = data.par().map(Some).into_optional().cloned().collect();
    ///
    /// assert_eq!(cloned, Some(vec!["a".to_string(), "b".to_string()]));
    /// ```
    fn cloned<'a, O>(
        self,
    ) -> impl ParOption<
        Item = O,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, FnCloned<'a, O>>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        Self: ParOption<Item = &'a O>,
        O: Clone + 'a,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParOptionIter::new(iter, x1, x2.mapped(FnCloned::new()), exe, params)
    }

    // transformations

    /// Maps each successful element with closure `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = (1..4).into_par().map(Some).into_optional().map(|x| 2 * x).collect();
    /// assert_eq!(out, Some(vec![2, 4, 6]));
    /// ```
    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = Q,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = Self::Size,
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
    /// let out: Option<Vec<_>> = (1..5)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .inspect(|_| {
    ///         seen.fetch_add(1, Ordering::Relaxed);
    ///     })
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 2, 3, 4]));
    /// assert_eq!(seen.load(Ordering::Relaxed), 4);
    /// ```
    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = Self::Item,
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
    /// let out: Option<Vec<_>> = (1..7)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .filter(|x| x % 2 == 1)
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 3, 5]));
    /// ```
    fn filter<H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = Self::Item,
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
    /// let out: Option<Vec<_>> = ["1", "x", "5"]
    ///     .into_par()
    ///     .map(|s| Some(s))
    ///     .into_optional()
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 5]));
    /// ```
    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = Q,
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
    /// let out: Option<Vec<_>> = (1..4)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .flat_map(|x| [x, x + 10])
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 11, 2, 12, 3, 13]));
    /// ```
    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = V::Item,
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
    /// let out: Option<Vec<_>> = nested.into_par().map(Some).into_optional().flatten().collect();
    ///
    /// assert_eq!(out, Some(vec![1, 2, 3, 4]));
    /// ```
    fn flatten(
        self,
    ) -> impl ParOption<
        Item = <Self::Item as IntoIterator>::Item,
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
    /// - `None` if computation short-circuits due to a failure (`None` element)
    /// - `Some(None)` if no successful element exists
    /// - `Some(Some(x))` for the first successful element
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!((1..4).into_par().map(Some).into_optional().first(), Some(Some(1)));
    /// assert_eq!(Vec::<usize>::new().into_par().map(Some).into_optional().first(), Some(None));
    /// assert_eq!(vec![None, Some(1), Some(3)].into_par().into_optional().first(), None);
    /// ```
    fn first(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Send;

    /// Reduces successful items into one value using `f`.
    ///
    /// Returns:
    /// - `None` if computation short-circuits due to a failure
    /// - `Some(None)` if there is no successful value to reduce
    /// - `Some(Some(x))` for the reduced value
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = (1..6).into_par().map(Some).into_optional().reduce(|a, b| a + b);
    /// assert_eq!(ok, Some(Some(15)));
    ///
    /// let fail = vec![Some(1), None, Some(3)].into_par().into_optional().reduce(|a, b| a + b);
    /// assert_eq!(fail, None);
    /// ```
    fn reduce<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send;

    /// Collects successful items into `dst`.
    ///
    /// Returns `None` if any element fails, `Some(())` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut dst = vec![10usize];
    /// let ok = (0..3).into_par().map(Some).into_optional().collect_into(&mut dst);
    /// assert_eq!(ok, Some(()));
    /// assert_eq!(dst, vec![10, 0, 1, 2]);
    ///
    /// let mut dst_fail = vec![];
    /// let fail = vec![Some(1usize), None, Some(3)]
    ///     .into_par()
    ///     .into_optional()
    ///     .collect_into(&mut dst_fail);
    /// assert_eq!(fail, None);
    /// ```
    fn collect_into<C>(self, dst: &mut C) -> Option<()>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    /// Collects successful items into a new collection.
    ///
    /// Returns `None` if any element fails, otherwise `Some(collection)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok: Option<Vec<_>> = (1..4).into_par().map(Some).into_optional().collect();
    /// assert_eq!(ok, Some(vec![1, 2, 3]));
    ///
    /// let fail: Option<Vec<_>> = vec![Some(1), None, Some(3)].into_par().into_optional().collect();
    /// assert_eq!(fail, None);
    /// ```
    fn collect<C>(self) -> Option<C>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    // compute - derived

    /// Returns `Some(true)` if all successful items satisfy `f`.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!((1..5).into_par().map(Some).into_optional().all(|x| x > &0), Some(true));
    /// assert_eq!((1..5).into_par().map(Some).into_optional().all(|x| x % 2 == 0), Some(false));
    /// assert_eq!(vec![Some(1), None, Some(3)].into_par().into_optional().all(|x| x > &0), None);
    /// ```
    fn all<F>(self, f: F) -> Option<bool>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.map(|x| f(&x)).find(|x| !*x).map(|x| x.is_none())
    }

    /// Returns `Some(true)` if any successful item satisfies `f`.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!((1..5).into_par().map(Some).into_optional().any(|x| x % 2 == 0), Some(true));
    /// assert_eq!((1..5).into_par().map(Some).into_optional().any(|x| x > &10), Some(false));
    /// assert_eq!(vec![Some(1), None, Some(3)].into_par().into_optional().any(|x| x % 2 == 0), None);
    /// ```
    fn any<F>(self, f: F) -> Option<bool>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.map(|x| f(&x)).find(|x| *x).map(|x| x.is_some())
    }

    /// Counts successful elements.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = (1..11)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .filter(|x| x % 3 == 0)
    ///     .count();
    /// assert_eq!(ok, Some(3));
    ///
    /// let fail = vec![Some(1usize), None, Some(3)].into_par().into_optional().count();
    /// assert_eq!(fail, None);
    /// ```
    fn count(self) -> Option<usize> {
        self.map(|_| 1).reduce(|a, b| a + b).map(|x| x.unwrap_or(0))
    }

    /// Finds first (ordered) or any (arbitrary) successful item satisfying `f`.
    ///
    /// Equivalent to `self.filter(f).first()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let found = (1..101)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .find(|x| x % 17 == 0);
    /// assert_eq!(found, Some(Some(17)));
    ///
    /// let fail = vec![Some(1usize), None, Some(34)].into_par().into_optional().find(|x| x % 17 == 0);
    /// assert_eq!(fail, None);
    /// ```
    fn find<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.filter(&f).first()
    }

    /// Folds successful elements into per-thread accumulators.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let partials = (1..6)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .fold(|| 0usize, |acc, x| *acc += x);
    /// assert_eq!(partials.as_ref().map(|v| v.iter().sum::<usize>()), Some(15));
    ///
    /// let fail = vec![Some(1usize), None, Some(3)]
    ///     .into_par()
    ///     .into_optional()
    ///     .fold(|| 0usize, |acc, x| *acc += x);
    /// assert_eq!(fail, None);
    /// ```
    fn fold<B, I, F>(self, init: I, f: F) -> Option<Vec<B>>
    where
        B: Send,
        I: Fn() -> B + Sync,
        F: Fn(&mut B, Self::Item) + Copy + Send,
    {
        let mut use_vec = UseVec::new(|_| init());
        let par_use = self.use_vec(&mut use_vec);
        let result = par_use.for_each(move |u: &mut B, x| f(u, x));
        result.map(|_| use_vec.into_vec())
    }

    /// Executes `f` for each successful element.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::{AtomicUsize, Ordering};
    /// use orx_parallel::*;
    ///
    /// let total = AtomicUsize::new(0);
    /// let ok = (1..5)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .for_each(|x| {
    ///         total.fetch_add(x, Ordering::Relaxed);
    ///     });
    ///
    /// assert_eq!(ok, Some(()));
    /// assert_eq!(total.load(Ordering::Relaxed), 10);
    /// ```
    fn for_each<F>(self, f: F) -> Option<()>
    where
        F: Fn(Self::Item) + Send + Copy,
    {
        self.map(f).reduce(|_, _| {}).map(|_| ())
    }

    /// Returns maximum successful element.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!((1..5).into_par().map(Some).into_optional().max(), Some(Some(4)));
    /// assert_eq!(Vec::<usize>::new().into_par().map(Some).into_optional().max(), Some(None));
    /// assert_eq!(vec![Some(1usize), None, Some(3)].into_par().into_optional().max(), None);
    /// ```
    fn max(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    /// Returns successful element considered maximum by comparator `f`.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .max_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Some(Some(5)));
    /// ```
    fn max_by<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match f(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns successful element with maximum key value.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .max_by_key(|x| x.abs());
    /// assert_eq!(x, Some(Some(-10)));
    /// ```
    fn max_by_key<B, F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns minimum successful element.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!((1..5).into_par().map(Some).into_optional().min(), Some(Some(1)));
    /// assert_eq!(Vec::<usize>::new().into_par().map(Some).into_optional().min(), Some(None));
    /// assert_eq!(vec![Some(1usize), None, Some(3)].into_par().into_optional().min(), None);
    /// ```
    fn min(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::min)
    }

    /// Returns successful element considered minimum by comparator `f`.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .min_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Some(Some(-10)));
    /// ```
    fn min_by<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match f(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Returns successful element with minimum key value.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .min_by_key(|x| x.abs());
    /// assert_eq!(x, Some(Some(0)));
    /// ```
    fn min_by_key<B, F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Sums successful elements using [`Sum`] implementation.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok: Option<usize> = (1..5).into_par().map(Some).into_optional().sum();
    /// assert_eq!(ok, Some(10));
    ///
    /// let fail: Option<usize> = vec![Some(1usize), None, Some(3)].into_par().into_optional().sum();
    /// assert_eq!(fail, None);
    /// ```
    fn sum<S>(self) -> Option<S>
    where
        Self::Item: Sum<S>,
        S: Send,
    {
        self.map(Self::Item::owned)
            .reduce(Self::Item::add)
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }
}
