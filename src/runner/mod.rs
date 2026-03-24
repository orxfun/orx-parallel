use crate::pool::ParThreadPool;

pub trait Runner {
    type Pool: ParThreadPool;

    /// Reference to the underlying thread pool.
    fn thread_pool(&self) -> &Self::Pool;

    /// Mutable reference to the underlying thread pool.
    fn thread_pool_mut(&mut self) -> &mut Self::Pool;
}
