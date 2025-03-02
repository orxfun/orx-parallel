mod basic;
mod parallel_runner;
mod parallel_runner_to_arch;
mod thread_runner;

pub use basic::BasicRunner;
pub use parallel_runner_to_arch::ParallelRunnerToArchive;

pub type DefaultRunner = BasicRunner;
