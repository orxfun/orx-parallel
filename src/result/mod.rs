#[cfg(test)]
mod tests;

mod par;
mod par_iter;
mod par_iter_core;
mod par_runner;
mod thread_execution;

pub use par::ParRes;
pub use par_iter::ParResIter;
pub use par_iter_core::ParResIterCore;
pub use par_runner::ParRunnerRes;
