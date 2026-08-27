use crate::parameters::{NumThreads, Params};
use core::num::NonZeroUsize;

/// Abstraction for parallel execution environments and thread pool management.
///
/// `ParThreadPool` defines how parallel computations are executed on a set of worker threads.
/// Any type implementing this trait can serve as a thread pool for orx-parallel computations.
///
/// # Thread Count Decision
///
/// The actual number of threads used in a computation is determined by combining multiple
/// configuration layers:
///
/// 1. **Pool Layer** (`max_num_threads()`) - The thread pool's maximum capacity
/// 2. **Environment Layer** (`ORX_NUM_THREADS`) - Global limit from environment variable
/// 3. **Computation Layer** (`.num_threads()` on Par) - Per-computation request
/// 4. **Input Size** - Cannot exceed the number of input elements
///
/// The `max_num_threads_for_computation()` method implements this logic by returning the
/// minimum of all these constraints.
///
/// # Example
///
/// ```ignore
/// use orx_parallel::*;
///
/// // Pool setup: 8 threads requested, but env limits to 4
/// // ORX_NUM_THREADS=4 is set
/// let pool = Pool::once(8);  // pool.max_num_threads() == 4
///
/// // Computation: request 6 threads on 100-element input
/// let result: Vec<_> = (0..100)
///     .into_par()
///     .map(|x| x * 2)
///     .pool(pool)
///     .num_threads(6)
///     .collect();
/// // Result: min(min(6, 100), 4) = 4 threads used
/// ```
///
/// # Implementations
///
/// - [`Pool::once`](crate::Pool::once) - Lightweight virtual pool, spawns threads on-demand
/// - [`BasicPool`](crate::BasicPool) - Persistent thread pool
/// - Rayon thread pools via `Pool::rayon()`
///
/// See the [`thread_usage.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/thread_usage.md) documentation for a complete guide.
pub trait ParThreadPool {
    /// Scope type of the thread pool.
    type ScopeRef<'s, 'env, 'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    /// Executes the `work` within scope `s`.
    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env;

    /// Executes the scoped computation `f`.
    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(Self::ScopeRef<'s, 'env, 'scope>) + Send;

    /// Returns the maximum number of threads available in the pool.
    ///
    /// This value reflects all constraints applied up to pool creation time, including:
    /// - The requested thread count from pool construction
    /// - The `ORX_NUM_THREADS` environment variable (if set)
    /// - The system's available CPU cores
    ///
    /// Individual computations can further limit this via `.num_threads()` method.
    fn max_num_threads(&self) -> NonZeroUsize;

    // provided

    /// Calculates the actual thread count for a computation considering multiple constraints.
    ///
    /// This method implements the core thread count decision logic by combining:
    ///
    /// 1. **Pool constraint** (`self.max_num_threads()`)
    ///    - The thread pool's maximum capacity
    ///    - Already includes environment variable constraints
    ///
    /// 2. **Computation constraint** (`params.num_threads`)
    ///    - Per-computation request from `.num_threads()` method
    ///    - Can be `NumThreads::Auto` (use all available)
    ///    - Or `NumThreads::Max(n)` (hard limit)
    ///
    /// 3. **Input size constraint** (known upper bound from `size_hint.1`)
    ///    - Cannot spawn more threads than input elements
    ///    - When input size is unknown (None), this constraint doesn't apply
    ///
    /// # Returns
    ///
    /// The minimum of all constraints, representing the actual thread count to use.
    ///
    /// # Decision Logic
    ///
    /// ```text
    /// let available = self.max_num_threads()           // Pool limit
    ///
    /// let requested = match (size_hint.1, params.num_threads) {
    ///     (Some(len), Auto) => min(len, MaxUsize),     // Cap by input size
    ///     (Some(len), Max(n)) => min(len, n),          // Cap by input size and request
    ///     (None, Auto) => MaxUsize,                    // No constraints
    ///     (None, Max(n)) => n,                         // Only respect request
    /// };
    ///
    /// return min(requested, available)                 // Final decision
    /// ```
    ///
    /// # Parameters
    ///
    /// - `params` - Contains `.num_threads` setting from `.num_threads()` method
    /// - `size_hint` - Tuple of (lower_bound, Option<upper_bound>) for input size
    ///   - If upper_bound is `None`, input size is unknown
    ///   - If upper_bound is `Some(n)`, input has at most n elements
    fn max_num_threads_for_computation(
        &self,
        params: Params,
        size_hint: (usize, Option<usize>),
    ) -> usize {
        let ava = self.max_num_threads();

        let req = match (size_hint.1, params.num_threads) {
            (Some(len_ub), NumThreads::Auto) => NonZeroUsize::new(len_ub.max(1)).expect(">0"),
            (Some(len_ub), NumThreads::Max(nt)) => {
                NonZeroUsize::new(len_ub.max(1)).expect(">0").min(nt)
            }
            (None, NumThreads::Auto) => NonZeroUsize::MAX,
            (None, NumThreads::Max(nt)) => nt,
        };

        core::cmp::min(req, ava).into()
    }
}
