#[cfg(feature = "std")]
use crate::{NumThreads, pool::pool_impl::StdDefaultPool};

pub struct Pool;

impl Pool {
    #[cfg(feature = "std")]
    pub fn once(num_threads: impl Into<NumThreads>) -> StdDefaultPool {
        StdDefaultPool::new(num_threads)
    }
}
