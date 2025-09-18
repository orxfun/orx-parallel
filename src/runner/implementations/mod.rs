#[cfg(test)]
mod tests;

mod sequential;
pub use sequential::SequentialRunner;

#[cfg(feature = "std")]
mod std_runner;
#[cfg(feature = "std")]
pub use std_runner::StdRunner;

#[cfg(feature = "poolite")]
mod poolite;
#[cfg(feature = "poolite")]
pub use poolite::RunnerWithPoolitePool;

#[cfg(feature = "rayon")]
mod rayon;
#[cfg(feature = "rayon")]
pub use rayon::RunnerWithRayonPool;

#[cfg(feature = "scoped-pool")]
mod scoped_pool;
#[cfg(feature = "scoped-pool")]
pub use scoped_pool::RunnerWithScopedPool;

#[cfg(feature = "scoped_threadpool")]
mod scoped_threadpool;
#[cfg(feature = "scoped_threadpool")]
pub use scoped_threadpool::RunnerWithScopedThreadPool;
