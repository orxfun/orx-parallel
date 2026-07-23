#[cfg(any(feature = "std", feature = "rayon-core"))]
use crate::NumThreads;
#[cfg(all(feature = "wasm-web-threads", target_arch = "wasm32"))]
use crate::pool::pool_impl::WasmWebPool;
#[cfg(all(feature = "wasm-web-threads-experimental", target_arch = "wasm32"))]
use crate::pool::pool_impl::WasmWebPoolExp;
#[cfg(feature = "std")]
use crate::{BasicPool, pool::pool_impl::OncePool};

/// Factory for creating thread pools with different characteristics.
///
/// `Pool` provides builder methods to create various types of thread pools that can be used
/// for parallel computations. Each pool type has different properties regarding thread lifecycle
/// and persistence.
///
/// > **Note:** `Pool` is a convenience factory for thread pools provided or adapted by this crate.
/// > You can also implement [`ParThreadPool`](crate::ParThreadPool) yourself and pass it directly
/// > to `.pool(...)` or to runner constructors that accept any thread pool implementing the trait.
///
/// # Thread Count Configuration
///
/// When creating a pool, the thread count is determined by combining:
///
/// 1. **Requested count** - Passed to factory methods
/// 2. **Environment limit** - `ORX_PARALLEL_MAX_NUM_THREADS` if set
/// 3. **System availability** - Number of logical CPUs available
///
/// The pool will use the minimum of these constraints.
///
/// # Examples
///
/// ```ignore
/// use orx_parallel::*;
///
/// // Create a OncePool with auto-detection (subject to ORX_PARALLEL_MAX_NUM_THREADS)
/// let pool = Pool::once(NumThreads::Auto);
///
/// // Create a OncePool capped at 4 threads
/// let pool = Pool::once(4);  // Converted from usize via From impl
///
/// // Create a persistent BasicPool with 8 threads
/// let pool = Pool::basic(8);
///
/// // Create a Rayon pool (requires rayon-core feature)
/// let pool = Pool::rayon(NumThreads::Auto)?;
/// ```
///
/// # Pool Types
///
/// - **OncePool** (default) - Spawns threads only when needed, releases after computation
/// - **BasicPool** - Maintains persistent workers across multiple computations
/// - **Rayon** - Uses the Rayon parallel runtime (external crate)
///
/// See the [`threading_model.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/threading_model.md) documentation for complete details.
pub struct Pool;

impl Pool {
    /// Creates a lightweight on-demand pool with the specified thread configuration.
    ///
    /// A `OncePool` is a lightweight virtual pool that spawns worker threads just before
    /// a computation starts and releases them immediately after. This reduces overhead when
    /// a persistent thread pool isn't needed.
    ///
    /// # Thread Count Decision
    ///
    /// The actual thread count is determined by:
    /// - The `num_threads` parameter
    /// - The `ORX_PARALLEL_MAX_NUM_THREADS` environment variable (if set)
    /// - The number of available system CPU cores
    ///
    /// The minimum of these constraints will be used.
    ///
    /// # Parameters
    ///
    /// - `num_threads` - Either:
    ///   - `0` or `NumThreads::Auto` - Use all available threads (respecting constraints)
    ///   - `n > 0` or `NumThreads::Max(n)` - Cap at `n` threads (respecting constraints)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use orx_parallel::*;
    ///
    /// // Auto-detect threads
    /// let pool = Pool::once(NumThreads::Auto);
    ///
    /// // Cap at 4 threads
    /// let pool = Pool::once(4);
    ///
    /// // Same as above (usize converts via From impl)
    /// let pool = Pool::once(NumThreads::Max(std::num::NonZeroUsize::new(4).unwrap()));
    /// ```
    ///
    /// # Default Behavior
    ///
    /// This is the default pool used when the "std" feature is enabled.
    /// Applications do not need to explicitly create an `OncePool` unless they want
    /// custom thread configuration.
    #[cfg(feature = "std")]
    pub fn once(num_threads: impl Into<NumThreads>) -> OncePool {
        OncePool::new(num_threads)
    }

    /// Creates a [`BasicPool`] with the specified thread configuration.
    ///
    /// A `BasicPool` maintains persistent worker threads that remain alive across
    /// multiple parallel computations. This is more efficient than `OncePool` when
    /// running many parallel operations sequentially.
    ///
    /// # Thread Count Decision
    ///
    /// Thread count is determined the same way as [`Self::once`]:
    /// - The `num_threads` parameter
    /// - The `ORX_PARALLEL_MAX_NUM_THREADS` environment variable (if set)
    /// - Available system CPU cores
    ///
    /// The minimum of these constraints will be used.
    ///
    /// # Parameters
    ///
    /// - `num_threads` - Configuration as described in [`Self::once`]
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use orx_parallel::*;
    ///
    /// // Create and reuse a persistent pool
    /// let pool = Pool::basic(8);
    ///
    /// for data in datasets {
    ///     let result = data.into_par()
    ///         .map(|x| process(x))
    ///         .pool(pool)
    ///         .collect();
    /// }
    /// ```
    ///
    /// # Benefits Over OncePool
    ///
    /// - Worker threads persist between computations
    /// - Avoids overhead of repeated thread spawning
    /// - Ideal for applications with many parallel tasks
    #[cfg(feature = "std")]
    pub fn basic(num_threads: impl Into<NumThreads>) -> BasicPool {
        BasicPool::new(num_threads)
    }

    /// Creates a Rayon [`ThreadPool`](https://docs.rs/rayon-core/latest/rayon_core/struct.ThreadPool.html).
    ///
    /// This method integrates with the Rayon parallel runtime. Rayon pools can be used
    /// with orx-parallel parallel iterators through the `.pool()` method.
    ///
    /// # Thread Count Decision
    ///
    /// Rayon's thread count is determined similarly to other pools:
    /// - When `num_threads` is `0` or `NumThreads::Auto`:
    ///   - Rayon uses `RAYON_NUM_THREADS` environment variable if set
    ///   - Otherwise uses the number of logical CPUs
    /// - When `num_threads` is `n > 0` or `NumThreads::Max(n)`:
    ///   - Rayon will start at most `n` threads
    ///
    /// Note: `ORX_PARALLEL_MAX_NUM_THREADS` is not automatically applied to Rayon pools.
    /// See Rayon documentation for its configuration options.
    ///
    /// # Parameters
    ///
    /// - `num_threads` - Configuration for the Rayon thread pool
    ///
    /// # Returns
    ///
    /// - `Ok(ThreadPool)` - Successfully created Rayon pool
    /// - `Err(ThreadPoolBuildError)` - Failed to create pool (e.g., invalid configuration)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use orx_parallel::*;
    ///
    /// // Create a Rayon pool with automatic thread detection
    /// let pool = Pool::rayon(NumThreads::Auto)?;
    ///
    /// // Create a Rayon pool capped at 4 threads
    /// let pool = Pool::rayon(4)?;
    ///
    /// let result = (0..1000)
    ///     .into_par()
    ///     .map(|x| x * 2)
    ///     .pool(pool)
    ///     .collect();
    /// ```
    ///
    /// # Features
    ///
    /// Requires the `rayon-core` feature to be enabled.
    ///
    /// [`ThreadPool`]: https://docs.rs/rayon-core/latest/rayon_core/struct.ThreadPool.html
    #[cfg(feature = "rayon-core")]
    pub fn rayon(
        num_threads: impl Into<NumThreads>,
    ) -> Result<rayon_core::ThreadPool, rayon_core::ThreadPoolBuildError> {
        let num_threads = match num_threads.into() {
            NumThreads::Auto => 0,
            NumThreads::Max(nt) => nt.into(),
        };
        rayon_core::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
    }

    /// Creates a wasm web-thread pool adapter backed by Rayon's global runtime.
    ///
    /// This pool is intended for `wasm32` web builds where the global Rayon pool
    /// is initialized by `wasm-bindgen-rayon`.
    ///
    /// # Parameters
    ///
    /// - `num_threads` - Desired maximum number of threads for computations:
    ///   - `NumThreads::Auto` uses Rayon's maximum supported thread count
    ///   - `NumThreads::Max(n)` caps usage at `n`
    ///
    /// # Features
    ///
    /// Requires `wasm-web-threads-experimental` feature and `wasm32` target.
    #[cfg(all(feature = "wasm-web-threads-experimental", target_arch = "wasm32"))]
    pub fn wasm_web_exp(num_threads: impl Into<NumThreads>) -> WasmWebPoolExp {
        WasmWebPoolExp::new(num_threads)
    }

    /// Creates the main wasm web-thread pool adapter.
    ///
    /// This pool is intended for `wasm32` web builds and uses the simplified wasm backend.
    ///
    /// # Parameters
    ///
    /// - `num_threads` - Desired maximum number of threads for computations:
    ///   - `NumThreads::Auto` uses backend default supported thread count
    ///   - `NumThreads::Max(n)` caps usage at `n`
    ///
    /// # Features
    ///
    /// Requires `wasm-web-threads` feature and `wasm32` target.
    #[cfg(all(feature = "wasm-web-threads", target_arch = "wasm32"))]
    pub fn wasm_web(num_threads: impl Into<NumThreads>) -> WasmWebPool {
        WasmWebPool::new(num_threads)
    }
}
