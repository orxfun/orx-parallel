#[cfg(test)]
mod tests;

mod par;
mod par_core;
mod par_iter;
mod par_runner;
mod thread_execution;

pub use par::ParUseResult;
pub use par_core::ParUseResultCore;
pub use par_iter::ParUseResultIter;
pub use par_runner::ParRunnerUseRes;
