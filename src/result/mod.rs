#[cfg(feature = "long-tests")]
#[cfg(test)]
mod tests;

mod par;
mod par_core;
mod par_iter;
mod par_runner;
mod thread_execution;

pub use par::ParResult;
pub use par_core::ParResultCore;
pub use par_iter::ParResultIter;
pub use par_runner::ParRunnerRes;
