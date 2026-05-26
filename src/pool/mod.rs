mod env;
mod new_pool;
mod par_thread_pool;
mod pool_impl;

#[cfg(feature = "std")]
pub use env::max_num_threads_by_env_variable;
pub use par_thread_pool::ParThreadPool;
#[cfg(feature = "std")]
pub use pool_impl::SimplePool;

#[cfg(feature = "std")]
pub type DefaultPool = pool_impl::StdDefaultPool;
#[cfg(not(feature = "std"))]
pub type DefaultPool = pool_impl::SequentialPool;
