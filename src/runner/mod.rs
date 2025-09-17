mod computation_kind;
mod implementations;
mod num_spawned;
mod parallel_runner;

pub(crate) use parallel_runner::{SharedStateOf, ThreadRunnerOf};

pub use crate::runner::implementations::StdRunner;
pub use computation_kind::ComputationKind;
pub use num_spawned::NumSpawned;
pub use parallel_runner::ParallelRunner;

pub type DefaultRunner = StdRunner;
