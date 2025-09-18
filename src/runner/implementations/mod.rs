#[cfg(test)]
mod tests;

mod runner_with_pool;
pub use runner_with_pool::RunnerWithPool;

mod sequential;
pub use sequential::SequentialPool;

#[cfg(feature = "std")]
mod std_runner;
#[cfg(feature = "std")]
pub use std_runner::StdDefaultPool;

#[cfg(feature = "pond")]
mod pond;
#[cfg(feature = "pond")]
pub use pond::PondPool;

#[cfg(feature = "poolite")]
mod poolite;

#[cfg(feature = "rayon-core")]
mod rayon_core;

#[cfg(feature = "scoped-pool")]
mod scoped_pool;

#[cfg(feature = "scoped_threadpool")]
mod scoped_threadpool;

#[cfg(feature = "yastl")]
mod yastl;
#[cfg(feature = "yastl")]
pub use yastl::YastlPool;
