#[cfg(test)]
mod tests;

mod par;
mod par_core;
mod par_iter;
mod par_runner;
mod thread_execution;

pub use par::ParOption;
pub use par_core::ParOptionCore;
pub use par_iter::ParOptionIter;
pub use par_runner::ParRunnerOpt;
