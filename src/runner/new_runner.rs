use crate::ParThreadPool;
use crate::runner::runner_variants::FixedChunkRunner;
#[cfg(feature = "std")]
use crate::runner::runner_variants::RunnerB;

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
    /// let result: Vec<_> = par.collect();
    /// ```
    pub fn fixed_chunk<P: ParThreadPool>(pool: P) -> FixedChunkRunner<P> {
        FixedChunkRunner::new(pool)
    }

    /// Creates runner-b placeholder strategy.
    ///
    /// Current implementation always uses chunk size 1.
    /// Requires the `std` feature.
    #[cfg(feature = "std")]
    pub fn b<P: ParThreadPool>(pool: P) -> RunnerB<P> {
        RunnerB::new(pool)
    }
}
