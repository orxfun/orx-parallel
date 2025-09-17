mod computation_kind;
mod fixed_chunk_executor;
mod parallel_executor;
pub(crate) mod parallel_runner_compute;
mod thread_executor;
mod thread_runner_compute;

pub use computation_kind::ComputationKind;
pub use parallel_executor::ParallelExecutor;
pub use thread_executor::ThreadExecutor;

/// Default parallel executor.
pub type DefaultExecutor = fixed_chunk_executor::FixedChunkRunner;
