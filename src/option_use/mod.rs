#[cfg(test)]
mod tests;

mod par;
mod par_core;
mod par_iter;
mod par_runner;
mod thread_execution;

pub use par::ParUseOption;
pub use par_core::ParUseOptionCore;
pub use par_iter::ParUseOptionIter;
pub use par_runner::ParRunnerUseOpt;
