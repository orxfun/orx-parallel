mod computation_kind;
mod implementations;
mod num_spawned;
mod parallel_runner;

pub(crate) use parallel_runner::{SharedStateOf, ThreadRunnerOf};

pub use computation_kind::ComputationKind;
pub use num_spawned::NumSpawned;
pub use parallel_runner::ParallelRunner;

pub type DefaultRunner = StdRunner;

#[cfg(feature = "std")]
pub use implementations::StdRunner;

#[cfg(feature = "rayon")]
pub use implementations::RunnerWithRayonPool;

#[cfg(feature = "scoped_threadpool")]
pub use implementations::RunnerWithScopedThreadPool;
