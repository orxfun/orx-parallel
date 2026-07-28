use crate::ParThreadPool;
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
///
/// #[cfg(feature = "std")]
/// let pool = Pool::once(4);
///
/// #[cfg(feature = "std")]
/// let par = par.runner(Runner::fixed_chunk(pool));
///
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
    ///
    /// #[cfg(feature = "std")]
    /// let pool = Pool::once(4);
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner(Runner::fixed_chunk(pool));
    ///
    /// let result: Vec<_> = par.collect();
    /// ```
    pub fn fixed_chunk<P: ParThreadPool>(pool: P) -> FixedChunkRunner<P> {
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
    /// let pool = Pool::once(4);
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner(Runner::adaptive(pool));
    ///
    /// let result: Vec<_> = par.collect();
    /// ```
    #[cfg(feature = "std")]
    pub fn adaptive<P: ParThreadPool>(pool: P) -> AdaptiveChunkRunner<P> {
        AdaptiveChunkRunner::new(pool)
    }
}
