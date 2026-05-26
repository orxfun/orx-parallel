#[cfg(feature = "std")]
use crate::{NumThreads, pool::pool_impl::OncePool};

pub struct Pool;

impl Pool {
    #[cfg(feature = "std")]
    pub fn once(num_threads: impl Into<NumThreads>) -> OncePool {
        OncePool::new(num_threads)
    }
}
