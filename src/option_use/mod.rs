#[cfg(test)]
mod tests;

mod par;
mod par_iter;
mod par_iter_core;
mod par_runner;
mod size_pairs;
mod thread_execution;

pub use par::ParUseOpt;
pub use par_iter::ParUseOptIter;
pub use par_iter_core::ParUseOptIterCore;
pub use par_runner::ParRunnerUseOpt;
pub use size_pairs::SizePairUseOpt;
