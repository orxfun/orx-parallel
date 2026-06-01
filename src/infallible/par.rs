use crate::common_par_traits::ParInfCommon;
use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::xap::FlattenOf;
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, ParIter};
use crate::infallible::{Xap, xap_variants::Id};
use crate::infallible_use::{ParUseIter, xap_variants::IdUse};
use crate::option::ParOptionIter;
use crate::pool::ParThreadPool;
use crate::result::ParResultIter;
use crate::sizes::Size;
use crate::use_var::{UseFun, UseSlice, UseVec};
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParOption, ParResult, ParUse, Sum,
};
use crate::{infallible::par_core::ParCore, runner::ParRunner};
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Infallible parallel iterator.
///
/// `Par` is the central trait for describing parallel computations as iterator
/// pipelines. It mirrors common sequential iterator operations (`map`,
/// `filter`, `flat_map`, `collect`, `reduce`, ...) while allowing runtime
/// configuration of execution details such as number of threads, chunk size,
/// iteration order, and runner/pool selection.
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// let sum_of_even_squares: usize = (1..11)
///     .into_par()
///     .map(|x| x * x)
///     .filter(|x| x % 2 == 0)
///     .sum();
///
/// assert_eq!(sum_of_even_squares, 220);
/// ```
pub trait Par: Sized + ParCore + ParInfCommon<CommonItem = Self::Item> {
    // configuration

    /// Replaces the current parallel runner with `runner`.
    ///
    /// This allows per-computation control over execution strategy.
    ///
    /// Please see [`Runner`] for parallel runners implemented in this crate.
    ///
    /// [`Runner`]: crate::Runner
    ///
    /// # Examples
    ///
    /// ```
    /// #![cfg(feature = "std")]
    ///
    /// use orx_parallel::*;
    ///
    /// let baseline: usize = (0..1000).into_par().sum();
    ///
    /// let configured: usize = (0..1000)
    ///     .into_par()
    ///     .runner(Runner::fixed_chunk(Pool::once(4)))
    ///     .sum();
    ///
    /// assert_eq!(baseline, configured);
    /// ```
    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl Par<Item = Self::Item, Xap = Self::Xap, Input = Self::Input>;

    /// Wraps the current parallel runner with a diagnostics-enabled runner.
    ///
    /// The returned iterator behaves the same, but additionally reports runtime
    /// diagnostics at the end of the computation.
    ///
    /// # Examples
    ///
    /// ```
    /// #![cfg(feature = "std")]
    ///
    /// use orx_parallel::*;
    ///
    /// let sum = (1..100_001)
    ///     .into_par()
    ///     .num_threads(4)
    ///     .runner_with_diagnostics()
    ///     .sum();
    ///
    /// assert_eq!(sum, 5000050000);
    /// ```
    ///
    /// This will print a summary report which currently looks like the following:
    ///
    /// ```console
    /// │ # Parallel Executor Diagnostics
    /// │
    /// │   Available threads : 4
    /// │   Used threads      : 4
    /// │   Wall time         : 1.15 ms
    /// │
    /// │ ## Summary Table
    /// │   thread  num_chunks   num_tasks  min_chunk  avg_chunk  max_chunk    util%
    /// │   ------  ----------  ----------  ---------  ---------  ---------  -------
    /// │        0          35       27335        781        781        781   100.0%
    /// │        1          32       24992        781        781        781    91.5%
    /// │        2          30       23430        781        781        781    85.9%
    /// │        3          28       21868        781        781        781    77.8%
    /// │
    /// │ ## Workload Balance
    /// │   max/min task ratio  : 1.25x  (1.00 = perfect balance)
    /// │   coeff. of variation : 8.3%  (lower is better)
    /// │
    /// │ ## Thread Active Timeline  (each block ≈ 0.02 ms)
    /// │   [ 0] ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇
    /// │   [ 1]     ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇
    /// │   [ 2]         ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇
    /// │   [ 3]             ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇
    /// │
    /// │ ## Thread Task Distribution  (bar length ∝ tasks processed)
    /// │   [ 0] ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇  (27335)
    /// │   [ 1] ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇  (24992)
    /// │   [ 2] ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇  (23430)
    /// │   [ 3] ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇  (21868)
    /// ```
    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl Par<Item = Self::Item, Xap = Self::Xap, Input = Self::Input>;

    /// Replaces the pool used by the current runner.
    ///
    /// Please see [`Pool`] for thread pools that can be used for parallel computations.
    ///
    /// [`Pool`]: crate::Pool
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// #[cfg(feature = "std")]
    /// {
    ///     let sum: usize = (1..101)
    ///         .into_par()
    ///         .pool(Pool::once(4))
    ///         .sum();
    ///     assert_eq!(sum, 5050);
    /// }
    ///
    /// #[cfg(feature = "rayon-core")]
    /// {
    ///     let sum: usize = (1..101)
    ///         .into_par()
    ///         .pool(Pool::rayon(8))
    ///         .sum();
    ///     assert_eq!(sum, 5050);
    /// }
    /// ```
    fn pool<P: ParThreadPool>(
        self,
        pool: P,
    ) -> impl Par<Item = Self::Item, Xap = Self::Xap, Input = Self::Input>;

    /// Sets the maximum number of worker threads for this computation.
    ///
    /// Integer values map as follows:
    /// - `0` => automatic (default)
    /// - `n > 0` => at most `n` threads
    ///
    /// This allows per-computation control resource usage.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let sum: usize = (1..11).into_par().num_threads(1).sum();
    /// assert_eq!(sum, 55);
    /// ```
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
    /// let values: Vec<_> = (0..32)
    ///     .into_par()
    ///     .chunk_size(8)
    ///     .map(|x| x + 1)
    ///     .collect();
    ///
    /// assert_eq!(values.len(), 32);
    /// assert_eq!(values[0], 1);
    /// assert_eq!(values[31], 32);
    /// ```
    ///
    /// # Rules of Thumb
    ///
    /// * Automatic chunk size (default) is efficient in general.
    ///   Parallel runner aims to find best chunk sizes to balance between minimizing parallelization overhead
    ///   and maximizing resource utilization.
    /// * While tuning a specific computation, we aim to find the smallest chunk size that is large enough
    ///   to mitigate the impact of parallelization overhead.
    /// * If the individual tasks are large enough, parallelization overhead becomes insignificant making
    ///   `chunk_size = 1` the optimal choice.
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets iteration order semantics for operations sensitive to ordering.
    ///
    /// `Ordered` (default) preserves positional meaning (for example, `first` returns the
    /// earliest matching element in input order). `Arbitrary` allows any matching
    /// element that is reached first in parallel execution.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ordered = (1..10_000)
    ///     .into_par()
    ///     .iteration_order(IterationOrder::Ordered)
    ///     .find(|x| x % 3421 == 0);
    /// assert_eq!(ordered, Some(3421));
    ///
    /// let any = (1..10_000)
    ///     .into_par()
    ///     .iteration_order(IterationOrder::Arbitrary)
    ///     .find(|x| x % 3421 == 0)
    ///     .unwrap();
    /// assert!([3421, 6842].contains(&any));
    /// ```
    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    /// Converts `Par<Item = Option<T>>` into `ParOption<Item = T>`.
    ///
    /// The resulting fallible iterator **short-circuits** to `None` if any element is `None`.
    ///
    /// Similar to pattern using the `?` operator, fallible iterators allow us to work with
    /// the **success path**.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok: Option<Vec<_>> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<i32>().ok())
    ///     .into_optional()
    ///     .map(|x| x * 2)
    ///     .filter(|x| *x > 3)
    ///     .collect();
    /// assert_eq!(ok, Some(vec![4, 6]));
    ///
    /// let fail: Option<Vec<_>> = ["1", "x", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<i32>().ok())
    ///     .into_optional()
    ///     .map(|x| x * 2)
    ///     .filter(|x| *x > 3)
    ///     .collect();
    /// assert_eq!(fail, None);
    /// ```
    ///
    /// Notice that `x` is of type `i32`, rather than `Option<i32>`, which allows for concise
    /// expressions.
    ///
    /// Without fallible iterators, the above result could be obtained by the following version,
    /// which is not only more verbose, but also lacks the short-circuiting mechanism.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok: Option<Vec<_>> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<i32>().ok())
    ///     .map(|x| x.map(|x| x * 2))
    ///     .filter(|x| x.as_ref().map(|x| *x > 3).unwrap_or(true))
    ///     .collect::<Vec<_>>()
    ///     .into_iter()
    ///     .collect();
    /// assert_eq!(ok, Some(vec![4, 6]));
    /// ```
    fn into_optional<T>(
        self,
    ) -> impl ParOption<
        Item = T,
        Xap1 = Self::Xap,
        M = T,
        Xap2 = Id<T>,
        Input = Self::Input,
        Size = <<Self::Xap as Xap>::Size as Size>::IntoPair,
    >
    where
        Self::Xap: Xap<O = Option<T>>,
    {
        let (iter, xap, exe, params) = self.destruct();
        let x = ParOptionIter::new(iter, xap, Id::new(), exe, params);
        x
    }

    /// Converts `Par<Item = Result<T, E>>` into `ParResult<Item = T, Error = E>`.
    ///
    /// The resulting fallible iterator **short-circuits** and returns the first
    /// observed error.
    ///
    /// Similar to pattern using the `?` operator, fallible iterators allow us to
    /// work with the **success path**.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok: Result<Vec<_>, _> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<i32>())
    ///     .into_fallible()
    ///     .map(|x| x * 2)
    ///     .filter(|x| *x > 3)
    ///     .collect();
    /// assert_eq!(ok, Ok(vec![4, 6]));
    ///
    /// let fail: Result<Vec<_>, _> = ["1", "x", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<i32>())
    ///     .into_fallible()
    ///     .map(|x| x * 2)
    ///     .filter(|x| *x > 3)
    ///     .collect();
    /// assert!(fail.is_err());
    /// ```
    ///
    /// Notice that `x` is of type `i32`, rather than `Result<i32, _>`, which
    /// allows for concise expressions.
    ///
    /// Without fallible iterators, the above result could be obtained by the
    /// following version, which is not only more verbose, but also lacks the
    /// short-circuiting mechanism.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok: Result<Vec<_>, _> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<i32>())
    ///     .map(|x| x.map(|x| x * 2))
    ///     .filter(|x| x.as_ref().map(|x| *x > 3).unwrap_or(true))
    ///     .collect::<Vec<_>>()
    ///     .into_iter()
    ///     .collect();
    /// assert_eq!(ok, Ok(vec![4, 6]));
    /// ```
    fn into_fallible<T, E>(
        self,
    ) -> impl ParResult<
        Item = T,
        Error = E,
        Xap1 = Self::Xap,
        M = T,
        Xap2 = Id<T>,
        Input = Self::Input,
        Size = <<Self::Xap as Xap>::Size as Size>::IntoPair,
    >
    where
        Self::Xap: Xap<O = Result<T, E>>,
    {
        let (iter, xap, exe, params) = self.destruct();
        ParResultIter::new(iter, xap, Id::new(), exe, params)
    }

    /// Creates one mutable `Use` value per participating worker.
    ///
    /// The initializer `f` is called with the worker's thread index, and the
    /// returned value is then passed as `&mut Use` to downstream [`ParUse`]
    /// operations such as `map`, `filter`, `flat_map`, `reduce`, and `for_each`.
    ///
    /// This is useful for thread-local scratch buffers, counters, or other
    /// mutable state that should not be shared across workers.
    ///
    /// # Examples
    ///
    /// Reusing one buffer per worker avoids allocating a fresh String
    /// for every parsed item.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let values: Vec<_> = (1..4)
    ///     .into_par()
    ///     .num_threads(1)
    ///     .use_new(|_| String::new())
    ///     .map(|buffer, x| {
    ///         buffer.clear();
    ///         buffer.push_str(&x.to_string());
    ///         buffer.parse::<usize>().unwrap() * 10
    ///     })
    ///     .collect();
    ///
    /// assert_eq!(values, vec![10, 20, 30]);
    /// ```
    ///
    /// Some pipelines need fast worker-local randomness, for example for
    /// sampling, randomized search, or simulation.
    /// Seeding one RNG per worker with thread_idx creates independent
    /// thread-local random streams without shared mutable state.
    ///
    /// ```
    /// use orx_parallel::*;
    /// use rand::{Rng, SeedableRng};
    /// use rand_chacha::ChaCha8Rng;
    ///
    /// let values: Vec<_> = (0..8)
    ///     .into_par()
    ///     .num_threads(2)
    ///     .use_new(|thread_idx| ChaCha8Rng::seed_from_u64(thread_idx as u64 + 1))
    ///     .map(|rng, _| rng.random_range(0..100usize))
    ///     .collect();
    ///
    /// assert_eq!(values.len(), 8);
    /// assert!(values.into_iter().all(|x| x < 100));
    /// ```
    fn use_new<U, F>(
        self,
        f: F,
    ) -> impl ParUse<Item = Self::Item, Use = U, Xap = IdUse<Self::Xap, U>, Input = Self::Input>
    where
        U: Send,
        F: Fn(usize) -> U + Sync,
    {
        let (iter, xap, exe, params) = self.destruct();
        let xap = IdUse::new(xap);
        let using = UseFun::new(f);
        ParUseIter::new(using, iter, xap, exe, params)
    }

    /// Uses an externally owned [`UseVec`] as worker-local mutable state.
    ///
    /// Unlike [`Par::use_new`], the state container is provided by the caller,
    /// which allows reading back per-worker values after the computation.
    ///
    /// This is practical when we need thread-local accumulation with a final
    /// merge step, such as per-thread partial sums or local metrics.
    ///
    /// Note that the resulting `UseVec` length equals the number of worker
    /// threads that actually participated in the computation. Exactly one
    /// element is created per participating thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let n = 10_000usize;
    /// let mut use_vec = UseVec::new(|_| 0usize);
    ///
    /// (0..n)
    ///     .into_par()
    ///     .map(|x| 2 * x)
    ///     .use_vec(&mut use_vec)
    ///     .for_each(|thread_sum, x| *thread_sum += x);
    ///
    /// let partial_sums = use_vec.into_vec();
    /// let total: usize = partial_sums.into_iter().sum();
    ///
    /// assert_eq!(total, (n - 1) * n);
    /// ```
    ///
    /// The following example demonstrates an expensive per-thread state:
    /// a pre-allocated scratch buffer.
    ///
    /// ```
    /// use core::fmt::Write;
    /// use core::sync::atomic::{AtomicUsize, Ordering};
    /// use orx_parallel::*;
    ///
    /// let created = AtomicUsize::new(0);
    /// let mut use_vec = UseVec::new(|_| {
    ///     created.fetch_add(1, Ordering::Relaxed);
    ///     String::with_capacity(4096)
    /// });
    ///
    /// let out: Vec<_> = (0..64)
    ///     .into_par()
    ///     .num_threads(4)
    ///     .use_vec(&mut use_vec)
    ///     .map(|buffer, x| {
    ///         buffer.clear();
    ///         write!(buffer, "{x}").unwrap();
    ///         buffer.parse::<usize>().unwrap()
    ///     })
    ///     .collect();
    ///
    /// assert_eq!(out, (0..64).collect::<Vec<_>>());
    ///
    /// let buffers = use_vec.into_vec();
    /// assert_eq!(created.load(Ordering::Relaxed), buffers.len());
    /// assert!(buffers.len() <= 4);
    /// ```
    fn use_vec<U, F>(
        self,
        use_vec: &mut UseVec<U, F>,
    ) -> impl ParUse<Item = Self::Item, Use = U, Xap = IdUse<Self::Xap, U>, Input = Self::Input>
    where
        U: Send,
        F: Fn(usize) -> U + Sync,
    {
        let (iter, xap, exe, params) = self.destruct();
        let xap = IdUse::new(xap);
        ParUseIter::new(use_vec, iter, xap, exe, params)
    }

    /// Uses a caller-provided mutable slice as worker-local mutable state.
    ///
    /// This is similar to [`Par::use_vec`], but the state storage is a
    /// borrowed slice instead of an owned `UseVec`. Therefore, no per-thread
    /// state objects are created by this method; existing slice elements are
    /// reused as thread-local state.
    ///
    /// The number of worker threads that can participate in the computation is
    /// limited by `slice.len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let n = 10_000usize;
    /// let mut thread_sums = vec![0usize; 4];
    ///
    /// (0..n)
    ///     .into_par()
    ///     .num_threads(16) // actual participating workers are limited to 4
    ///     .map(|x| 2 * x)
    ///     .use_slice(&mut thread_sums)
    ///     .for_each(|thread_sum, x| *thread_sum += x);
    ///
    /// let total: usize = thread_sums.into_iter().sum();
    /// assert_eq!(total, (n - 1) * n);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `slice` is empty.
    fn use_slice<'a, U>(
        self,
        slice: &'a mut [U],
    ) -> impl ParUse<Item = Self::Item, Use = U, Xap = IdUse<Self::Xap, U>, Input = Self::Input>
    where
        U: Sync + 'a,
    {
        assert!(
            slice.len() > 0,
            "Number of parallel threads is limited to slice.len(); and hence, slice cannot be empty."
        );
        let (iter, xap, exe, params) = self.destruct();
        let xap = IdUse::new(xap);
        let using = UseSlice::new(slice);
        ParUseIter::new(using, iter, xap, exe, params)
    }

    /// Copies elements of a reference iterator.
    ///
    /// Equivalent to `.map(|&x| x)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec![1, 2, 3];
    /// let copied: Vec<_> = data.par().copied().collect();
    ///
    /// assert_eq!(copied, vec![1, 2, 3]);
    /// ```
    fn copied<'a, O>(
        self,
    ) -> impl Par<Item = O, Xap = MappedOf<Self::Xap, FnCopied<'a, O>>, Input = Self::Input>
    where
        Self: Par<Item = &'a O>,
        O: Copy + 'a,
    {
        let (iter, xap, exe, params) = self.destruct();
        ParIter::new(iter, xap.mapped(FnCopied::new()), exe, params)
    }

    /// Clones elements of a reference iterator.
    ///
    /// Equivalent to `.map(|x| x.clone())`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec!["a".to_string(), "b".to_string()];
    /// let cloned: Vec<_> = data.par().cloned().collect();
    ///
    /// assert_eq!(cloned, vec!["a".to_string(), "b".to_string()]);
    /// ```
    fn cloned<'a, O>(
        self,
    ) -> impl Par<Item = O, Xap = MappedOf<Self::Xap, FnCloned<'a, O>>, Input = Self::Input>
    where
        Self: Par<Item = &'a O>,
        O: Clone + 'a,
    {
        let (iter, xap, exe, params) = self.destruct();
        ParIter::new(iter, xap.mapped(FnCloned::new()), exe, params)
    }

    // transformations

    /// Maps each element with closure `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let doubled: Vec<_> = (1..4).into_par().map(|x| 2 * x).collect();
    /// assert_eq!(doubled, vec![2, 4, 6]);
    /// ```
    fn map<Q, H>(
        self,
        h: H,
    ) -> impl Par<Item = Q, Xap = MapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    /// Runs `h` on each element and forwards the item unchanged.
    ///
    /// Useful for logging or debugging pipelines.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Vec<_> = (1..5)
    ///     .into_par()
    ///     .inspect(|x| {
    ///         println!("observed {x}");
    ///     })
    ///     .collect();
    ///
    /// assert_eq!(out, vec![1, 2, 3, 4]);
    /// ```
    fn inspect<H>(
        self,
        h: H,
    ) -> impl Par<Item = Self::Item, Xap = InsOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&Self::Item) + Copy + Send;

    /// Keeps only elements satisfying predicate `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let odds: Vec<_> = (1..7).into_par().filter(|x| x % 2 == 1).collect();
    /// assert_eq!(odds, vec![1, 3, 5]);
    /// ```
    fn filter<H>(
        self,
        h: H,
    ) -> impl Par<Item = Self::Item, Xap = FilOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&Self::Item) -> bool + Copy + Send;

    /// Maps and filters in a single pass.
    ///
    /// Returns mapped values for elements where `h` returns `Some(_)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let nums: Vec<_> = ["1", "x", "5"]
    ///     .into_par()
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(nums, vec![1, 5]);
    /// ```
    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl Par<Item = Q, Xap = FilMapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send;

    /// Maps each element to an iterator and flattens one level.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Vec<_> = (1..4).into_par().flat_map(|x| [x, x + 10]).collect();
    /// assert_eq!(out, vec![1, 11, 2, 12, 3, 13]);
    /// ```
    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl Par<Item = V::Item, Xap = FlatMapOf<Self::Xap, V, H>, Input = Self::Input>
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send;

    /// Flattens one level of nested iterables.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let nested = vec![vec![1, 2], vec![3, 4]];
    /// let flat: Vec<_> = nested.into_par().flatten().collect();
    ///
    /// assert_eq!(flat, vec![1, 2, 3, 4]);
    /// ```
    fn flatten(
        self,
    ) -> impl Par<
        Item = <Self::Item as IntoIterator>::Item,
        Xap = FlattenOf<Self::Xap>,
        Input = Self::Input,
    >
    where
        Self::Item: IntoIterator;

    // compute

    /// Returns the first item according to iteration order, or `None` if empty.
    ///
    /// With `IterationOrder::Ordered` (default), this is the earliest matching item by
    /// input position. With `IterationOrder::Arbitrary`, this may be any
    /// matching item reached first in parallel execution.
    ///
    /// This operation is short-circuiting: once a first candidate is determined,
    /// remaining work is cancelled.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!(Vec::<usize>::new().into_par().first(), None);
    /// assert_eq!((1..4).into_par().first(), Some(1));
    /// ```
    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send;

    /// Reduces items into one value using associative reducer `f`.
    ///
    /// Returns `None` for an empty iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let reduced = (1..6).into_par().reduce(|a, b| a + b);
    /// assert_eq!(reduced, Some(15));
    /// ```
    fn reduce<F>(self, f: F) -> Option<Self::Item>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send;

    /// Collects all items into `dst`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut dst = vec![10];
    /// (0..3).into_par().collect_into(&mut dst);
    /// assert_eq!(dst, vec![10, 0, 1, 2]);
    /// ```
    fn collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    /// Collects all items into a new collection.
    ///
    /// When a flat structure is not required, collecting into [`Vec2`] might lead to significant
    /// improvements in certain scenarios. Note that `Vec2<T>` is simply `Vec<Vec<T>>` with at most
    /// _number of threads_ inner vectors.
    ///
    /// [`Vec2`]: crate::Vec2
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Vec<_> = (1..4).into_par().map(|x| x * 2).collect();
    /// assert_eq!(out, vec![2, 4, 6]);
    /// ```
    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    // compute - derived

    /// Returns `true` if all items satisfy predicate `f`.
    ///
    /// Empty iterators return `true`.
    ///
    /// This operation is short-circuiting: evaluation stops as soon as one item
    /// fails the predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert!((1..5).into_par().all(|x| x > &0));
    /// assert!(!(1..5).into_par().all(|x| x % 2 == 0));
    /// ```
    fn all<F>(self, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.map(|x| f(&x)).find(|x| *x == false).is_none()
    }

    /// Returns `true` if any item satisfies predicate `f`.
    ///
    /// Empty iterators return `false`.
    ///
    /// This operation is short-circuiting: evaluation stops as soon as one item
    /// satisfies the predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert!((1..5).into_par().any(|x| x % 2 == 0));
    /// assert!(!(1..5).into_par().any(|x| x > &10));
    /// ```
    fn any<F>(self, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.map(|x| f(&x)).find(|x| *x == true).is_some()
    }

    /// Counts elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let n = (1..11).into_par().filter(|x| x % 3 == 0).count();
    /// assert_eq!(n, 3);
    /// ```
    fn count(self) -> usize {
        self.map(|_| 1).reduce(|a, b| a + b).unwrap_or(0)
    }

    /// Finds first ([`Ordered`], default) or any ([`Arbitrary`]) item satisfying predicate `f`.
    ///
    /// This is equivalent to `self.filter(f).first()`.
    ///
    /// This operation is short-circuiting: once a matching item is found,
    /// remaining work is cancelled.
    ///
    /// [`Ordered`]: crate::IterationOrder::Ordered
    /// [`Arbitrary`]: crate::IterationOrder::Arbitrary
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let found = (1..101).into_par().find(|x| x % 17 == 0);
    /// assert_eq!(found, Some(17));
    /// ```
    fn find<F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.filter(&f).first()
    }

    /// Folds elements into per-thread accumulators and returns them.
    ///
    /// The output contains one accumulator for each participating worker.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let num_threads = 2;
    ///
    /// let partials: Vec<usize> = (1..6)
    ///     .into_par()
    ///     .num_threads(num_threads)
    ///     .fold(|| 0usize, |acc, x| *acc += x);
    ///
    /// assert!(partials.len() <= num_threads);
    ///
    /// assert_eq!(partials.iter().sum::<usize>(), 15);
    /// ```
    fn fold<B, I, F>(self, init: I, f: F) -> Vec<B>
    where
        B: Send,
        I: Fn() -> B + Sync,
        F: Fn(&mut B, Self::Item) + Copy + Send,
    {
        let mut use_vec = UseVec::new(|_| init());
        let par_use = self.use_vec(&mut use_vec);
        par_use.for_each(move |u: &mut B, x| f(u, x));
        use_vec.into_vec()
    }

    /// Executes `f` for each item.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::{AtomicUsize, Ordering};
    /// use orx_parallel::*;
    ///
    /// let total = AtomicUsize::new(0);
    ///
    /// (1..5)
    ///     .into_par()
    ///     .for_each(|x| {
    ///         total.fetch_add(x, Ordering::Relaxed);
    ///     });
    ///
    /// assert_eq!(total.load(Ordering::Relaxed), 10);
    /// ```
    fn for_each<F>(self, f: F)
    where
        F: Fn(Self::Item) + Send + Copy,
    {
        let _ = self.map(f).reduce(|_, _| {});
    }

    /// Returns maximum element, or `None` if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!((1..5).into_par().max(), Some(4));
    /// assert_eq!(Vec::<usize>::new().into_par().max(), None);
    /// ```
    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    /// Returns element considered maximum by comparator `f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .max_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Some(5));
    /// ```
    fn max_by<F>(self, f: F) -> Option<Self::Item>
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

    /// Returns element with maximum key value.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .max_by_key(|x| x.abs());
    /// assert_eq!(x, Some(-10));
    /// ```
    fn max_by_key<B, F>(self, f: F) -> Option<Self::Item>
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

    /// Returns minimum element, or `None` if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!((1..5).into_par().min(), Some(1));
    /// assert_eq!(Vec::<usize>::new().into_par().min(), None);
    /// ```
    fn min(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::min)
    }

    /// Returns element considered minimum by comparator `f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .min_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Some(-10));
    /// ```
    fn min_by<F>(self, f: F) -> Option<Self::Item>
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

    /// Returns element with minimum key value.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .min_by_key(|x| x.abs());
    /// assert_eq!(x, Some(0));
    /// ```
    fn min_by_key<B, F>(self, f: F) -> Option<Self::Item>
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

    /// Sums elements using [`Sum`] implementation of the item type.
    ///
    /// Empty iterators return additive identity (`zero`).
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let sum: usize = (1..5).into_par().sum();
    /// assert_eq!(sum, 10);
    /// ```
    fn sum<S>(self) -> S
    where
        Self::Item: Sum<S>,
        S: Send,
    {
        self.map(Self::Item::owned)
            .reduce(Self::Item::add)
            .unwrap_or(Self::Item::zero())
    }
}
