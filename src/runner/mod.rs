mod computation_kind;
mod implementations;
mod num_spawned;
mod parallel_runner;

pub(crate) use parallel_runner::{SharedStateOf, ThreadRunnerOf};

pub use computation_kind::ComputationKind;
pub use implementations::{RunnerWithPool, SequentialPool};
pub use num_spawned::NumSpawned;
pub use parallel_runner::ParallelRunner;

#[cfg(feature = "pond")]
pub use implementations::PondPool;

#[cfg(feature = "std")]
pub use implementations::StdDefaultPool;

#[cfg(feature = "yastl")]
pub use implementations::YastlPool;

// DEFAULT

/// Default runner used by orx-parallel computations:
///
/// * [`RunnerWithPool`] with [`StdDefaultPool`] when "std" feature is enabled,
/// * [`RunnerWithPool`] with `SequentialPool` otherwise.
#[cfg(feature = "std")]
pub type DefaultRunner = RunnerWithPool<StdDefaultPool>;
/// Default runner used by orx-parallel computations:
///
/// * [`RunnerWithPool`] with `StdDefaultPool` when "std" feature is enabled,
/// * [`RunnerWithPool`] with [`SequentialPool`] otherwise.
#[cfg(not(feature = "std"))]
pub type DefaultRunner = RunnerWithPool<SequentialPool>;
