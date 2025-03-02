mod basic;
mod parallel_runner;
mod thread_runner;

pub use basic::BasicRunner;
pub use parallel_runner::ParallelRunner;

pub type DefaultRunner = BasicRunner;
