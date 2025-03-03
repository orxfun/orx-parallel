mod basic_to_arch;
mod fixed_chunk_runner;
mod parallel_runner;
mod parallel_runner_to_arch;
mod thread_runner;

pub use basic_to_arch::BasicRunnerToArch;
pub use parallel_runner_to_arch::ParallelRunnerToArchive;

pub use fixed_chunk_runner::FixedChunkRunner;
pub use parallel_runner::ParallelRunner;

pub type DefaultRunner = FixedChunkRunner;
