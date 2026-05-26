#[cfg(feature = "std")]
use crate::pool::pool_impl::StdDefaultPool;

pub struct Pool;

impl Pool {
    #[cfg(feature = "std")]
    pub fn once(num_threads: usize) -> StdDefaultPool {
        // StdDefaultPool::n
        todo!()
    }
}
