mod computation_kind;
mod implementations;
mod num_spawned;
mod parallel_runner;

pub(crate) use parallel_runner::{SharedStateOf, ThreadRunnerOf};

pub use computation_kind::ComputationKind;
pub use num_spawned::NumSpawned;
pub use parallel_runner::ParallelRunner;

pub use implementations::SequentialRunner;

#[cfg(feature = "std")]
pub use implementations::StdRunner;

#[cfg(feature = "poolite")]
pub use implementations::RunnerWithPoolitePool;

#[cfg(feature = "rayon")]
pub use implementations::RunnerWithRayonPool;

#[cfg(feature = "scoped-pool")]
pub use implementations::RunnerWithScopedPool;

#[cfg(feature = "scoped_threadpool")]
pub use implementations::RunnerWithScopedThreadPool;

// DEFAULT

/// Default runner used by orx-parallel computations:
///
/// * [`StdRunner`] when "std" feature is enabled,
/// * `SequentialRunner` otherwise.
#[cfg(feature = "std")]
pub type DefaultRunner = StdRunner;
/// Default runner used by orx-parallel computations:
///
/// * `StdRunner` when "std" feature is enabled,
/// * [`SequentialRunner`] otherwise.
#[cfg(not(feature = "std"))]
pub type DefaultRunner = SequentialRunner;
