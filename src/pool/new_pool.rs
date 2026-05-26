#[cfg(feature = "std")]
use crate::{BasicPool, NumThreads, pool::pool_impl::OncePool};

pub struct Pool;

impl Pool {
    #[cfg(feature = "std")]
    pub fn once(num_threads: impl Into<NumThreads>) -> OncePool {
        OncePool::new(num_threads)
    }

    #[cfg(feature = "std")]
    pub fn basic(num_threads: impl Into<NumThreads>) -> BasicPool {
        BasicPool::new(num_threads)
    }

    /// Creates a rayon [`ThreadPool`].
    ///
    /// When `num_threads` is set to `n > 0` or `NumThreads::Max(n)`, then the resulting thread pools
    /// are guaranteed to start at most this number of threads.
    ///
    /// When `num_threads` is 0 or `NumThreads::Auto`, then the Rayon runtime will select the number
    /// of threads automatically. At present, this is based on the RAYON_NUM_THREADS environment variable
    /// (if set), or the number of logical CPUs (otherwise). In the future, however, the default behavior
    /// may change to dynamically add or remove threads as needed.
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
}
