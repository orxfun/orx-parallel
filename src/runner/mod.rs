mod computation_kind;
mod fixed_chunk_runner;
mod par_iter_with_runner;
mod parallel_runner;
mod thread_runner;

pub use computation_kind::ComputationKind;
pub use parallel_runner::ParallelRunner;
pub use thread_runner::ThreadRunner;

pub type DefaultRunner = fixed_chunk_runner::FixedChunkRunner;
