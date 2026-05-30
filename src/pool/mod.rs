mod env;
mod new_pool;
mod par_thread_pool;
mod pool_impl;

pub use new_pool::Pool;

pub use par_thread_pool::ParThreadPool;
#[cfg(feature = "std")]
pub use pool_impl::BasicPool;

#[cfg(feature = "std")]
pub type DefaultPool = pool_impl::OncePool;
#[cfg(not(feature = "std"))]
pub type DefaultPool = pool_impl::SequentialPool;
