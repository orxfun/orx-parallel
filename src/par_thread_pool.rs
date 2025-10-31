use crate::{generic_values::runner_results::Fallibility, runner::NumSpawned};
use alloc::vec::Vec;
use core::num::NonZeroUsize;
use orx_concurrent_bag::ConcurrentBag;

/// A thread pool that can be used for parallel computation.
///
/// orx_parallel abstracts away the thread pool implementation and can work with different
/// thread pool implementations.
///
/// Parallel computation will not use any threads outside the pool.
/// Default std thread pool assumes all OS threads are available in the pool.
///
/// # Examples
///
/// ## Default std pool
///
/// **requires std feature**
///
/// Default parallel runner spawns scoped threads using `std::thread::scope`.
///
/// ```
/// use orx_parallel::*;
///
/// let sum = (0..1000).par().sum();
/// assert_eq!(sum, 1000 * 999 / 2);
///
/// // this is equivalent to
/// let sum = (0..1000).par().with_runner(DefaultRunner::default()).sum();
/// assert_eq!(sum, 1000 * 999 / 2);
/// ```
///
/// ## Rayon thread pool
///
/// **requires rayon feature**
///
/// The following example demonstrate using a rayon thread pool as the thread provider of
/// the parallel computation.
///
/// ```
/// use orx_parallel::*;
///
/// #[cfg(not(miri))]
/// #[cfg(feature = "rayon-core")]
/// {
///     let pool = rayon::ThreadPoolBuilder::new()
///         .num_threads(4)
///         .build()
///         .unwrap();
///
///     // creating a runner for the computation
///     let runner = RunnerWithPool::from(&pool);
///     let sum = (0..1000).par().with_runner(runner).sum();
///     assert_eq!(sum, 1000 * 999 / 2);
///
///     // or reuse a runner multiple times (identical under the hood)
///     let mut runner = RunnerWithPool::from(&pool);
///     let sum = (0..1000).par().with_runner(&mut runner).sum();
///     assert_eq!(sum, 1000 * 999 / 2);
/// }
/// ```
///
/// Note that since rayon::ThreadPool::scope only requires a shared reference `&self`,
/// we can concurrently create as many runners as we want from the same thread pool and use them concurrently.
///
/// ## Scoped thread pool
///
/// **requires scoped_threadpool feature**
///
/// The following example demonstrate using a scoped_threadpool thread pool as the thread provider of
/// the parallel computation.
///
/// ```
/// use orx_parallel::*;
///
/// #[cfg(not(miri))]
/// #[cfg(feature = "scoped_threadpool")]
/// {
///     // creating a runner for the computation
///     let mut pool = scoped_threadpool::Pool::new(4);
///     let runner = RunnerWithPool::from(&mut pool);
///     let sum = (0..1000).par().with_runner(runner).sum();
///     assert_eq!(sum, 1000 * 999 / 2);
///
///     // or reuse a runner multiple times (identical under the hood)
///     let mut pool = scoped_threadpool::Pool::new(4);
///     let mut runner = RunnerWithPool::from(&mut pool);
///     let sum = (0..1000).par().with_runner(&mut runner).sum();
///     assert_eq!(sum, 1000 * 999 / 2);
/// }
/// ```
///
/// Since scoped_thread_pool::Pool::scoped requires an exclusive reference `&mut self`,
/// we can create one runner from a pool at a time, note use of `&mut pool` in runner creation.
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
    fn max_num_threads(&self) -> NonZeroUsize;
}

// derived

pub trait ParThreadPoolCompute: ParThreadPool {
    fn map_in_pool<F, S, M, T>(
        &mut self,
        do_spawn: S,
        thread_map: M,
        max_num_threads: NonZeroUsize,
    ) -> (NumSpawned, Result<Vec<T>, F::Error>)
    where
        F: Fallibility,
        S: Fn(NumSpawned) -> bool + Sync,
        M: Fn(NumSpawned) -> Result<T, F::Error> + Sync,
        T: Send,
        F::Error: Send,
    {
        let thread_map = &thread_map;
        let mut nt = NumSpawned::zero();
        let thread_results = ConcurrentBag::with_fixed_capacity(max_num_threads.into());
        let bag = &thread_results;
        self.scoped_computation(|s| {
            while do_spawn(nt) {
                let num_spawned = nt;
                nt.increment();
                let work = move || {
                    bag.push(thread_map(num_spawned));
                };
                Self::run_in_scope(&s, work);
            }
        });

        let thread_results: Vec<_> = thread_results.into_inner().into();
        let result = F::reduce_results(thread_results);

        (nt, result)
    }

    fn run_in_pool<S, F>(&mut self, do_spawn: S, thread_do: F) -> NumSpawned
    where
        S: Fn(NumSpawned) -> bool + Sync,
        F: Fn(NumSpawned) + Sync,
    {
        let thread_do = &thread_do;
        let mut nt = NumSpawned::zero();
        self.scoped_computation(|s| {
            while do_spawn(nt) {
                let num_spawned = nt;
                nt.increment();
                let work = move || thread_do(num_spawned);
                Self::run_in_scope(&s, work);
            }
        });
        nt
    }
}

impl<X: ParThreadPool> ParThreadPoolCompute for X {}
