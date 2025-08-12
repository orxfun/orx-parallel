mod computation_kind;
mod fixed_chunk_runner;
mod parallel_runner;
pub(crate) mod parallel_runner_compute;
mod thread_next;
mod thread_runner;
mod thread_runner_compute;

pub use computation_kind::ComputationKind;
pub use parallel_runner::ParallelRunner;
pub(crate) use parallel_runner_compute::ParallelRunnerCompute;
pub(crate) use thread_next::ThreadNext;
pub use thread_runner::ThreadRunner;

/// Default parallel runner.
///
/// Unless explicitly set to another parallel runner by [`with_runner`] method,
/// parallel computations will be executed using the default parallel runner.
///
/// [`with_runner`]: crate::ParIter::with_runner
pub type DefaultRunner = fixed_chunk_runner::FixedChunkRunner;
