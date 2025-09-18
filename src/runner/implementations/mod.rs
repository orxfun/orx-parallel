#[cfg(test)]
mod tests;

mod sequential;
pub use sequential::SequentialRunner;

#[cfg(feature = "std")]
mod std_runner;
#[cfg(feature = "std")]
pub use std_runner::StdRunner;

#[cfg(feature = "pond")]
mod pond;
#[cfg(feature = "pond")]
pub use pond::{PondPool, RunnerWithPondPool};

#[cfg(feature = "poolite")]
mod poolite;
#[cfg(feature = "poolite")]
pub use poolite::RunnerWithPoolitePool;

#[cfg(feature = "rayon-core")]
mod rayon_core;
#[cfg(feature = "rayon-core")]
pub use rayon_core::RunnerWithRayonPool;

#[cfg(feature = "scoped-pool")]
mod scoped_pool;
#[cfg(feature = "scoped-pool")]
pub use scoped_pool::RunnerWithScopedPool;

#[cfg(feature = "scoped_threadpool")]
mod scoped_threadpool;
#[cfg(feature = "scoped_threadpool")]
pub use scoped_threadpool::RunnerWithScopedThreadPool;

#[cfg(feature = "yastl")]
mod yastl;
#[cfg(feature = "yastl")]
pub use yastl::{RunnerWithYastlPool, YastlPool};
