use crate::parameters::{NumThreads, Params, non_zero_or_one};
use crate::pools::scope::Scope;
use crate::pools::tasks::TaskQueue;
use core::num::NonZeroUsize;

/// Abstraction for parallel execution environments and thread pool management.
///
/// `ThreadPool` defines how parallel computations are executed on a set of worker threads.
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
/// - `Pool::basic` - Persistent thread pool (default)
/// - `Pool::once` - Lightweight virtual pool, spawns threads on-demand
/// - `Pool::rayon` - rayon thread pools
///
/// See the [`thread_usage.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/thread_usage.md) documentation for a complete guide.
pub trait ThreadPool {
    /// Scope type of the thread pool.
    type ScopeRef<'s, 'env, 'scope>: Scope<'s, 'env, 'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    /// Executes the scoped computation `f`.
    fn scope<'env, 'scope, F>(&'env self, f: F)
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

    /// Runs all `tasks` in parallel on this pool.
    ///
    /// `tasks` is a statically typed [`Tasks`] queue: pushed tasks are stored inline,
    /// requiring no object safety, boxing or heap allocation. None of the tasks start
    /// running on [`push`]; they all start in parallel only when `run_all` is called.
    ///
    /// [`push`]: crate::pools::tasks::TaskQueue::push
    ///
    /// # Example
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use orx_parallel::pools::tasks::TaskQueue;
    ///
    /// let work_for = |n| std::thread::sleep(std::time::Duration::from_millis(n));
    ///
    /// let tasks = Tasks::new()
    ///     .push(|| {
    ///         work_for(90);
    ///         println!("t1 completes 4th");
    ///     })
    ///     .push(|| println!("t2 completes 1st"))
    ///     .push(|| {
    ///         work_for(10);
    ///         println!("t3 completes 2nd");
    ///     })
    ///     .push(|| {
    ///         work_for(50);
    ///         println!("t4 completes 3rd");
    ///     });
    ///
    /// Pool::global().run_all(tasks);
    ///
    /// // prints:
    /// // t2 completes 1st
    /// // t3 completes 2nd
    /// // t4 completes 3rd
    /// // t1 completes 4th
    /// ```
    ///
    /// Below is a more practical example: computing independent statistics over the same
    /// input concurrently and collecting the results:
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use orx_parallel::pools::tasks::TaskQueue;
    /// use std::sync::Mutex;
    ///
    /// let numbers = [4, 8, 15, 16, 23, 42];
    ///
    /// let sum = Mutex::new(0);
    /// let max = Mutex::new(i32::MIN);
    /// let all_positive = Mutex::new(false);
    ///
    /// let tasks = Tasks::new()
    ///     .push(|| *sum.lock().unwrap() = numbers.iter().sum())
    ///     .push(|| *max.lock().unwrap() = numbers.iter().copied().max().unwrap())
    ///     .push(|| *all_positive.lock().unwrap() = numbers.iter().all(|&x| x > 0));
    ///
    /// Pool::global().run_all(tasks);
    ///
    /// println!(
    ///     "sum={}, max={}, all_positive={}",
    ///     sum.into_inner().unwrap(),
    ///     max.into_inner().unwrap(),
    ///     all_positive.into_inner().unwrap(),
    /// );
    /// ```
    fn run_all(&self, tasks: impl TaskQueue + Send) {
        self.scope(|s| tasks.run_in_scope(s));
    }
}

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
pub fn max_num_threads_for_computation(
    pool: &impl ThreadPool,
    params: Params,
    size_hint: (usize, Option<usize>),
) -> usize {
    let ava = pool.max_num_threads();

    let req = match (size_hint.1, params.num_threads) {
        (Some(len_ub), NumThreads::Auto) => non_zero_or_one(len_ub),
        (Some(len_ub), NumThreads::Max(nt)) => non_zero_or_one(len_ub).min(nt),
        (None, NumThreads::Auto) => NonZeroUsize::MAX,
        (None, NumThreads::Max(nt)) => nt,
    };

    core::cmp::min(req, ava).into()
}
