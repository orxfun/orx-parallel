use crate::infallible::recursive::par_core::ParRecCore;
use crate::infallible::xap::FlattenOf;
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf};
use crate::runner::ParRunner;
use crate::{ChunkSize, IterationOrder, NumThreads};
use crate::{ParCollectInto, Sum};
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
/// Related traits:
/// - [`ParUse`](crate::ParUse) for worker-local mutable state,
/// - [`ParOption`](crate::ParOption) for `Option`-based fallibility,
/// - [`ParResult`](crate::ParResult) for `Result`-based fallibility.
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// // A small rooted tree represented as adjacency lists; node 0 is the root.
/// let children: Vec<Vec<usize>> = vec![vec![1, 2], vec![3, 4], vec![5], vec![], vec![], vec![]];
///
/// let sum_of_even_squares: usize = [0usize]
///     .into_par_rec(|node| children[*node].iter().copied())
///     .map(|x| x * x)
///     .filter(|x| x % 2 == 0)
///     .sum();
///
/// assert_eq!(sum_of_even_squares, 20);
/// ```
pub trait ParRec: Sized + ParRecCore {
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
    /// use orx_parallel::*;
    ///
    /// let children: Vec<Vec<usize>> = vec![vec![1, 2], vec![3, 4], vec![5], vec![], vec![], vec![]];
    ///
    /// let baseline: usize = [0usize]
    ///     .into_par_rec(|node| children[*node].iter().copied())
    ///     .sum();
    ///
    /// let par = [0usize].into_par_rec(|node| children[*node].iter().copied());
    ///
    /// let par = par.runner(Runner::fixed());
    ///
    /// let configured: usize = par.sum();
    /// assert_eq!(baseline, configured);
    /// ```
    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParRec<Item = Self::Item, Xap = Self::Xap, Input = Self::Input>;

    /// Wraps the current parallel runner with a diagnostics-enabled runner.
    ///
    /// The returned iterator behaves the same, but additionally reports runtime
    /// diagnostics at the end of the computation.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "std")]
    /// # fn main() {
    /// use orx_parallel::*;
    ///
    /// let par = [1i32]
    ///     .into_par_rec(|&x| (x < 10_000).then_some(x + 1))
    ///     .num_threads(4);
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner_with_diagnostics();
    ///
    /// let sum = par.sum::<i32>();
    /// assert_eq!(sum, 50005000);
    /// # }
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
    ) -> impl ParRec<Item = Self::Item, Xap = Self::Xap, Input = Self::Input>;

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
    /// let sum: usize = [1usize]
    ///     .into_par_rec(|&x| (x < 10).then_some(x + 1))
    ///     .num_threads(1)
    ///     .sum();
    /// assert_eq!(sum, 55);
    ///
    /// // Cap at 4 threads
    /// let sum: usize = [1usize]
    ///     .into_par_rec(|&x| (x < 1000).then_some(x + 1))
    ///     .num_threads(4)
    ///     .sum();
    ///
    /// // Auto: uses available threads (respects ORX_PARALLEL_MAX_NUM_THREADS)
    /// let sum: usize = [1usize]
    ///     .into_par_rec(|&x| (x < 10).then_some(x + 1))
    ///     .num_threads(0)
    ///     .sum();
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
    /// let sum: usize = [1usize]
    ///     .into_par_rec(|&x| (x < 1000).then_some(x + 1))
    ///     .pool(pool)
    ///     .num_threads(6)  // Request 6...
    ///     .sum();          // ...but only 4 are available
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
    /// let values: Vec<_> = [0usize]
    ///     .into_par_rec(|&x| (x < 31).then_some(x + 1))
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
    /// let ordered = [1i32]
    ///     .into_par_rec(|&x| (x < 9_999).then_some(x + 1))
    ///     .iteration_order(IterationOrder::Ordered)
    ///     .find(|x| x % 3421 == 0);
    /// assert_eq!(ordered, Some(3421));
    ///
    /// let any = [1i32]
    ///     .into_par_rec(|&x| (x < 9_999).then_some(x + 1))
    ///     .iteration_order(IterationOrder::Arbitrary)
    ///     .find(|x| x % 3421 == 0)
    ///     .unwrap();
    /// assert!([3421, 6842].contains(&any));
    /// ```
    fn iteration_order(self, collect: IterationOrder) -> Self;

    // transformations

    /// Maps each element with closure `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let doubled: Vec<_> = [1i32]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .map(|x| 2 * x)
    ///     .collect();
    /// assert_eq!(doubled, vec![2, 4, 6]);
    /// ```
    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParRec<Item = Q, Xap = MapOf<Self::Xap, Q, H>, Input = Self::Input>
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
    /// let out: Vec<_> = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
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
    ) -> impl ParRec<Item = Self::Item, Xap = InsOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&Self::Item) + Copy + Send;

    /// Keeps only elements satisfying predicate `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let odds: Vec<_> = [1i32]
    ///     .into_par_rec(|&x| (x < 6).then_some(x + 1))
    ///     .filter(|x| x % 2 == 1)
    ///     .collect();
    /// assert_eq!(odds, vec![1, 3, 5]);
    /// ```
    fn filter<H>(
        self,
        h: H,
    ) -> impl ParRec<Item = Self::Item, Xap = FilOf<Self::Xap, H>, Input = Self::Input>
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
    /// let numbers: Vec<_> = ["1", "x", "5"]
    ///     .into_par_rec(|_: &&str| None::<&str>)
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(numbers, vec![1, 5]);
    /// ```
    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParRec<Item = Q, Xap = FilMapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send;

    /// Maps each element to an iterator and flattens one level.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Vec<_> = [1i32]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .flat_map(|x| [x, x + 10])
    ///     .collect();
    /// assert_eq!(out, vec![1, 11, 2, 12, 3, 13]);
    /// ```
    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParRec<Item = V::Item, Xap = FlatMapOf<Self::Xap, V, H>, Input = Self::Input>
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
    /// let mut flat: Vec<_> = nested
    ///     .into_par_rec(|_: &Vec<i32>| None::<Vec<i32>>)
    ///     .flatten()
    ///     .collect();
    /// flat.sort();
    ///
    /// assert_eq!(flat, vec![1, 2, 3, 4]);
    /// ```
    fn flatten(
        self,
    ) -> impl ParRec<
        Item = <Self::Item as IntoIterator>::Item,
        Xap = FlattenOf<Self::Xap>,
        Input = Self::Input,
    >
    where
        Self::Item: IntoIterator;

    // compute

    /// Returns an item, or `None` if empty.
    ///
    /// The item returned by a recursive iterator is not defined by input or traversal order.
    /// `IterationOrder` does not impose an ordering guarantee on recursive iterators.
    ///
    /// This operation is short-circuiting: once a first candidate is determined,
    /// remaining work is cancelled.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let empty = Vec::<usize>::new().into_par_rec(|_: &usize| None::<usize>).first();
    /// assert_eq!(empty, None);
    ///
    /// let first = [1usize]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .first();
    /// assert!(first.is_some_and(|x| [1, 2, 3].contains(&x)));
    /// ```
    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync;

    /// Reduces items into one value using associative reducer `f`.
    ///
    /// Returns `None` for an empty iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let reduced = [1i32]
    ///     .into_par_rec(|&x| (x < 5).then_some(x + 1))
    ///     .reduce(|a, b| a + b);
    /// assert_eq!(reduced, Some(15));
    /// ```
    fn reduce<F>(self, f: F) -> Option<Self::Item>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync;

    /// Collects all items into `dst`.
    ///
    /// Items are appended in an unspecified order for recursive iterators.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut dst = vec![10];
    /// [0i32]
    ///     .into_par_rec(|&x| (x < 2).then_some(x + 1))
    ///     .collect_into(&mut dst);
    /// dst[1..].sort();
    /// assert_eq!(dst, vec![10, 0, 1, 2]);
    /// ```
    fn collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync;

    /// Collects all items into a new collection in an unspecified order.
    ///
    /// When a flat structure is not required, collecting into [`Vec2`] might lead to
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
    /// let out: Vec<_> = [1i32]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .map(|x| x * 2)
    ///     .collect();
    /// out.sort();
    /// assert_eq!(out, vec![2, 4, 6]);
    /// ```
    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync;

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
    /// assert!([1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .all(|x| x > &0));
    /// assert!(![1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .all(|x| x % 2 == 0));
    /// ```
    fn all<F>(self, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync,
    {
        self.map(|x| f(&x)).find(|x| !*x).is_none()
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
    /// assert!([1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .any(|x| x % 2 == 0));
    /// assert!(![1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .any(|x| x > &10));
    /// ```
    fn any<F>(self, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync,
    {
        self.map(|x| f(&x)).find(|x| *x).is_some()
    }

    /// Counts elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let n = [1i32]
    ///     .into_par_rec(|&x| (x < 10).then_some(x + 1))
    ///     .filter(|x| x % 3 == 0)
    ///     .count();
    /// assert_eq!(n, 3);
    /// ```
    fn count(self) -> usize
    where
        <Self::Input as IntoIterator>::Item: Send + Sync,
    {
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
    /// let found = [1i32]
    ///     .into_par_rec(|&x| (x < 100).then_some(x + 1))
    ///     .find(|x| x % 17 == 0);
    /// assert_eq!(found, Some(17));
    /// ```
    fn find<F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync,
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
    /// let partials: Vec<usize> = [1usize]
    ///     .into_par_rec(|&x| (x < 5).then_some(x + 1))
    ///     .num_threads(2)
    ///     .fold(|| 0usize, |acc, x| *acc += x);
    ///
    /// assert!(!partials.is_empty());
    ///
    /// assert_eq!(partials.iter().sum::<usize>(), 15);
    /// ```
    fn fold<B, I, F>(self, init: I, f: F) -> Vec<B>
    where
        B: Send + Sync,
        I: Fn() -> B + Sync,
        F: Fn(&mut B, Self::Item) + Copy + Send + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync;

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
    /// [1usize]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .for_each(|x| {
    ///         total.fetch_add(x, Ordering::Relaxed);
    ///     });
    ///
    /// assert_eq!(total.load(Ordering::Relaxed), 10);
    /// ```
    fn for_each<F>(self, f: F)
    where
        F: Fn(Self::Item) + Send + Copy,
        <Self::Input as IntoIterator>::Item: Send + Sync,
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
    /// let max = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .max();
    /// assert_eq!(max, Some(4));
    ///
    /// let empty = Vec::<usize>::new()
    ///     .into_par_rec(|_: &usize| None::<usize>)
    ///     .max();
    /// assert_eq!(empty, None);
    /// ```
    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
        <Self::Input as IntoIterator>::Item: Send + Sync,
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
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .max_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Some(5));
    /// ```
    fn max_by<F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync,
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
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .max_by_key(|x| x.abs());
    /// assert_eq!(x, Some(-10));
    /// ```
    fn max_by_key<B, F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync,
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
    /// let min = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .min();
    /// assert_eq!(min, Some(1));
    ///
    /// let empty = Vec::<usize>::new()
    ///     .into_par_rec(|_: &usize| None::<usize>)
    ///     .min();
    /// assert_eq!(empty, None);
    /// ```
    fn min(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
        <Self::Input as IntoIterator>::Item: Send + Sync,
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
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .min_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Some(-10));
    /// ```
    fn min_by<F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync,
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
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .min_by_key(|x| x.abs());
    /// assert_eq!(x, Some(0));
    /// ```
    fn min_by_key<B, F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync,
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
    /// let sum: usize = [1usize]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .sum();
    /// assert_eq!(sum, 10);
    /// ```
    fn sum<S>(self) -> S
    where
        Self::Item: Sum<S>,
        S: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync,
    {
        self.map(Self::Item::owned)
            .reduce(Self::Item::add)
            .unwrap_or(Self::Item::zero())
    }
}
