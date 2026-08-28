use crate::ParThreadPool;
use crate::pool::DefaultPool;
use crate::pool::get_global_pool;
#[cfg(feature = "std")]
use crate::runner::runner_variants::AdaptiveChunkRunner;
use crate::runner::runner_variants::FixedChunkRunner;

/// Entry point for creating parallel runners that control how work is distributed across threads.
///
/// A runner is passed to `.runner(...)` on a parallel iterator to select the execution strategy.
///
/// > **Note:** `Runner` is a convenience factory for the runners provided by this crate.
/// > You can also implement a compatible runner type yourself and pass it directly to `.runner(...)` —
/// > the transformation accepts any type that satisfies the trait.
///
/// # Examples
///
/// ```rust
/// use orx_parallel::*;
///
/// let par = (0..100).par().map(|x| x + 1);
/// let par = par.runner(Runner::fixed());
/// let sum = par.sum();
///
/// let par = (0..100).par().map(|x| x + 1);
/// #[cfg(feature = "std")]
/// let par = par.runner(Runner::adaptive());
/// let sum = par.sum();
/// ```
pub struct Runner;

impl Runner {
    /// Creates a runner that splits work into fixed-size chunks ahead of time.
    ///
    /// This is the default strategy: the input is divided into equal chunks, one per thread.
    /// It has low overhead and works well when tasks have uniform cost.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let par = (0..100).par().map(|x| x + 1);
    /// let par = par.runner(Runner::fixed());
    ///
    /// let result: Vec<_> = par.collect();
    /// ```
    pub fn fixed() -> FixedChunkRunner<DefaultPool> {
        FixedChunkRunner::new(get_global_pool())
    }

    /// Creates a fixed chunk runner backed by `pool`.
    ///
    /// Use this when a computation should use a specific thread pool instead of the global
    /// default pool. The returned runner keeps the fixed-size chunking strategy of
    /// [`Self::fixed`] while delegating execution to the provided pool.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let pool = Pool::basic(4);
    /// let par = (0..100).par().runner(Runner::fixed_with_pool(pool));
    ///
    /// let result: Vec<_> = par.collect();
    /// ```
    pub fn fixed_with_pool<P: ParThreadPool>(pool: P) -> FixedChunkRunner<P> {
        FixedChunkRunner::new(pool)
    }

    /// Creates an adaptive chunk runner.
    ///
    /// This strategy explores and selects chunk sizes based on observed runtime behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let par = (0..100).par().map(|x| x + 1);
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner(Runner::adaptive());
    ///
    /// let result: Vec<_> = par.collect();
    /// ```
    #[cfg(feature = "std")]
    pub fn adaptive() -> AdaptiveChunkRunner<DefaultPool> {
        AdaptiveChunkRunner::new(get_global_pool())
    }

    /// Creates an adaptive chunk runner backed by `pool`.
    ///
    /// Use this when a computation should combine a specific thread pool with adaptive chunk
    /// sizing. The returned runner keeps the adaptive strategy of [`Self::adaptive`] while
    /// delegating execution to the provided pool.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let pool = Pool::once(4);
    /// let par = (0..100).par().runner(Runner::adaptive_with_pool(pool));
    ///
    /// let result: Vec<_> = par.collect();
    /// ```
    #[cfg(feature = "std")]
    pub fn adaptive_with_pool<P: ParThreadPool>(pool: P) -> AdaptiveChunkRunner<P> {
        AdaptiveChunkRunner::new(pool)
    }
}
