use crate::ParThreadPool;
#[cfg(all(feature = "std", feature = "experimental"))]
use crate::runner::runner_variants::DynChunkRunner;
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
/// # #[cfg(not(feature = "std"))] fn main() {}
/// #[cfg(feature = "std")]
/// {
///     use orx_parallel::*;
///
///     let pool = Pool::once(4);
///     let sum: usize = (0..1000)
///         .into_par()
///         .runner(Runner::fixed_chunk(pool))
///         .map(|x| x * 2)
///         .sum();
/// }
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
    /// # #[cfg(not(feature = "std"))] fn main() {}
    /// #[cfg(feature = "std")]
    /// {
    ///     use orx_parallel::*;
    ///
    ///     let pool = Pool::once(4);
    ///     let result: Vec<_> = (0..100)
    ///         .into_par()
    ///         .runner(Runner::fixed_chunk(pool))
    ///         .map(|x| x + 1)
    ///         .collect();
    /// }
    /// ```
    pub fn fixed_chunk<P: ParThreadPool>(pool: P) -> FixedChunkRunner<P> {
        FixedChunkRunner::new(pool)
    }

    /// Creates a runner that adjusts chunk sizes dynamically at runtime.
    ///
    /// Threads request new chunks as they finish, which improves load balancing when
    /// tasks have variable cost. Requires the `std` and `experimental` features.
    ///
    /// # Example
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let pool = Pool::once(4);
    /// let result: Vec<_> = (0..100)
    ///     .into_par()
    ///     .runner(Runner::dynamic_chunk(pool))
    ///     .map(|x| x + 1)
    ///     .collect();
    /// ```
    #[cfg(all(feature = "std", feature = "experimental"))]
    pub fn dynamic_chunk<P: ParThreadPool>(pool: P) -> DynChunkRunner<P> {
        DynChunkRunner::new(pool)
    }
}
