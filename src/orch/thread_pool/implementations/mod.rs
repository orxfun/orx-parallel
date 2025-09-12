mod std_os_thread_pool;
mod std_scoped_threads;

#[cfg(feature = "threadpool")]
mod impl_threadpool;

#[cfg(feature = "scoped_threadpool")]
mod impl_scoped_threadpool;

pub use std_os_thread_pool::StdOsThreadPool;
