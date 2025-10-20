use crate::{
    DefaultExecutor, ParThreadPool, ParallelExecutor, ParallelExecutorWithDiagnostics,
    runner::ParallelRunner,
};
use core::marker::PhantomData;

/// Parallel runner with a given pool of type `P` and parallel executor of `R`.
///
/// A `RunnerWithPool` can always be created from owned `pool` implementing [`ParThreadPool`], but also from
/// * `&pool` in most cases,
/// * `&mut pool` in others.
///
/// Note that default parallel runner; i.e., [`DefaultRunner`] is:
/// * `RunnerWithPool<StdDefaultPool>` when "std" feature is enabled,
/// * `RunnerWithPool<SequentialPool>` when "std" feature is disabled.
///
/// [`DefaultRunner`]: crate::DefaultRunner
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// // parallel computation generic over parallel runner; and hence, the thread pool
/// fn run_with_runner<R: ParallelRunner>(runner: R, input: &[usize]) -> Vec<String> {
///     input
///         .par()
///         .with_runner(runner)
///         .flat_map(|x| [*x, 2 * x, x / 7])
///         .map(|x| x.to_string())
///         .collect()
/// }
///
/// let vec: Vec<_> = (0..42).collect();
/// let input = vec.as_slice();
///
/// // runs sequentially on the main thread
/// let runner = RunnerWithPool::from(SequentialPool);
/// let expected = run_with_runner(runner, input);
///
/// // uses native threads
/// let runner = RunnerWithPool::from(StdDefaultPool::default());
/// let result = run_with_runner(runner, input);
/// assert_eq!(&expected, &result);
///
/// // uses rayon-core ThreadPool with 8 threads
/// #[cfg(feature = "rayon-core")]
/// {
///     let pool = rayon_core::ThreadPoolBuilder::new()
///         .num_threads(8)
///         .build()
///         .unwrap();
///     let result = run_with_runner(RunnerWithPool::from(&pool), input);
///     assert_eq!(&expected, &result);
/// }
///
/// // uses scoped-pool Pool with 8 threads
/// #[cfg(feature = "scoped-pool")]
/// {
///     let pool = scoped_pool::Pool::new(8);
///     let result = run_with_runner(RunnerWithPool::from(&pool), input);
///     assert_eq!(&expected, &result);
/// }
///
/// // uses scoped_threadpool Pool with 8 threads
/// #[cfg(feature = "scoped_threadpool")]
/// {
///     let mut pool = scoped_threadpool::Pool::new(8);
///     let result = run_with_runner(RunnerWithPool::from(&mut pool), input); // requires &mut pool
///     assert_eq!(&expected, &result);
/// }
///
/// // uses yastl Pool wrapped as YastlPool with 8 threads
/// #[cfg(feature = "yastl")]
/// {
///     let pool = YastlPool::new(8);
///     let result = run_with_runner(RunnerWithPool::from(&pool), input);
///     assert_eq!(&expected, &result);
/// }
///
/// // uses pond Pool wrapped as PondPool with 8 threads
/// #[cfg(feature = "pond")]
/// {
///     let mut pool = PondPool::new_threads_unbounded(8);
///     let result = run_with_runner(RunnerWithPool::from(&mut pool), input); // requires &mut pool
///     assert_eq!(&expected, &result);
/// }
///
/// // uses poolite Pool with 8 threads
/// #[cfg(feature = "poolite")]
/// {
///     let pool = poolite::Pool::with_builder(poolite::Builder::new().min(8).max(8)).unwrap();
///     let result = run_with_runner(RunnerWithPool::from(&pool), input);
///     assert_eq!(&expected, &result);
/// }
/// ```
pub struct RunnerWithPool<P, R = DefaultExecutor>
where
    P: ParThreadPool,
    R: ParallelExecutor,
{
    pool: P,
    runner: PhantomData<R>,
}

impl<P, R> Default for RunnerWithPool<P, R>
where
    P: ParThreadPool + Default,
    R: ParallelExecutor,
{
    fn default() -> Self {
        Self {
            pool: Default::default(),
            runner: PhantomData,
        }
    }
}

impl<P: ParThreadPool> From<P> for RunnerWithPool<P, DefaultExecutor> {
    fn from(pool: P) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<P, R> RunnerWithPool<P, R>
where
    P: ParThreadPool,
    R: ParallelExecutor,
{
    /// Converts the runner into the wrapped underlying pool.
    ///
    /// Note that a `RunnerWithPool` can always be created from owned `pool`, but also from
    /// * `&pool` in most cases,
    /// * `&mut pool` in others.
    ///
    /// This function is only relevant when the runner is created from owned pool, in which case
    /// `into_inner_pool` can be used to get back ownership of the pool.
    ///
    /// # Example
    ///
    /// The following example demonstrates the use case for rayon-core thread pool; however, it
    /// holds for all thread pool implementations.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let vec: Vec<_> = (0..42).collect();
    /// let input = vec.as_slice();
    ///
    /// #[cfg(feature = "rayon-core")]
    /// {
    ///     let pool = rayon_core::ThreadPoolBuilder::new()
    ///         .num_threads(8)
    ///         .build()
    ///         .unwrap();
    ///
    ///     // create runner owning the pool
    ///     let mut runner = RunnerWithPool::from(pool);
    ///
    ///     // use runner, and hence the pool, in parallel computations
    ///     let sum = input.par().with_runner(&mut runner).sum();
    ///     let max = input.par().with_runner(&mut runner).max();
    ///     let txt: Vec<_> = input
    ///         .par()
    ///         .with_runner(&mut runner)
    ///         .map(|x| x.to_string())
    ///         .collect();
    ///
    ///     // get back ownership of the pool
    ///     let pool: rayon_core::ThreadPool = runner.into_inner_pool();
    /// }
    /// ```
    pub fn into_inner_pool(self) -> P {
        self.pool
    }

    /// Converts the runner into one using the [`ParallelExecutor`] `Q` rather than `R`.
    pub fn with_executor<Q: ParallelExecutor>(self) -> RunnerWithPool<P, Q> {
        RunnerWithPool {
            pool: self.pool,
            runner: PhantomData,
        }
    }

    /// Converts executor of this runner `R` into one with diagnostics; i.e.,`ParallelExecutorWithDiagnostics<R>`.
    ///
    /// Note that [`ParallelExecutorWithDiagnostics`] prints the diagnostics on the stdout. Therefore, it must
    /// only be used while testing a program, not in production.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// // normal execution
    ///
    /// let range = 0..4096;
    /// let sum = range
    ///     .par()
    ///     .map(|x| x + 1)
    ///     .filter(|x| x.is_multiple_of(2))
    ///     .sum();
    /// assert_eq!(sum, 1073774592);
    ///
    /// // execution with diagnostics
    ///
    /// let range = 0..4096;
    /// let sum = range
    ///     .par()
    ///     .with_runner(DefaultRunner::default().with_diagnostics())
    ///     .map(|x| x + 1)
    ///     .filter(|x| x.is_multiple_of(2))
    ///     .sum();
    /// assert_eq!(sum, 1073774592);
    ///
    /// // prints diagnostics, which looks something like the following:
    /// //
    /// // - Number of threads used = 5
    /// //
    /// // - [Thread idx]: num_calls, num_tasks, avg_chunk_size, first_chunk_sizes
    /// //   - [0]: 25, 1600, 64, [64, 64, 64, 64, 64, 64, 64, 64, 64, 64]
    /// //   - [1]: 26, 1664, 64, [64, 64, 64, 64, 64, 64, 64, 64, 64, 64]
    /// //   - [2]: 13, 832, 64, [64, 64, 64, 64, 64, 64, 64, 64, 64, 64]
    /// //   - [3]: 0, 0, 0, []
    /// //   - [4]: 0, 0, 0, []
    /// ```
    pub fn with_diagnostics(self) -> RunnerWithPool<P, ParallelExecutorWithDiagnostics<R>> {
        RunnerWithPool {
            pool: self.pool,
            runner: PhantomData,
        }
    }
}

impl<P, R> ParallelRunner for RunnerWithPool<P, R>
where
    P: ParThreadPool,
    R: ParallelExecutor,
{
    type Executor = R;

    type ThreadPool = P;

    fn thread_pool(&self) -> &Self::ThreadPool {
        &self.pool
    }

    fn thread_pool_mut(&mut self) -> &mut Self::ThreadPool {
        &mut self.pool
    }
}
