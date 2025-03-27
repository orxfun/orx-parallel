mod computation_kind;
mod fixed_chunk_runner;
mod parallel_runner;
mod parallel_task;
mod thread_runner;

pub use computation_kind::ComputationKind;
pub use parallel_runner::ParallelRunner;
pub use parallel_task::{ParallelTask, ParallelTaskWithIdx};
pub use thread_runner::ThreadRunner;

pub type DefaultRunner = fixed_chunk_runner::FixedChunkRunner;
