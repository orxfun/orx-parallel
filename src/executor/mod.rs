mod fixed_chunk_executor;
pub(crate) mod parallel_compute;
mod parallel_executor;
mod thread_compute;
mod thread_executor;

pub use parallel_executor::ParallelExecutor;
pub use thread_executor::ThreadExecutor;

/// Default parallel executor.
pub type DefaultExecutor = fixed_chunk_executor::FixedChunkRunner;
