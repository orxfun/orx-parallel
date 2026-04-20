#[cfg(test)]
mod tests;

mod par;
mod par_iter;
mod par_iter_core;
mod par_runner;
mod thread_execution;

pub use par::ParUseRes;
pub use par_iter::ParUseResIter;
pub use par_iter_core::ParUseResIterCore;
pub use par_runner::ParRunnerUseRes;
