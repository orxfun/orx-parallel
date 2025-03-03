#[cfg(test)]
mod tests;

mod fixed_chunk_runner;
mod parallel_runner;
mod thread_runner;

pub use fixed_chunk_runner::FixedChunkRunner;
pub use parallel_runner::ParallelRunner;

pub type DefaultRunner = FixedChunkRunner;
