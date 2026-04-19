#[cfg(test)]
mod tests;

mod par;
mod par_iter;
mod par_runner;
mod size_pairs;
mod thread_execution;

pub use par::ParOpt;
pub use par_runner::ParRunnerOpt;
pub use size_pairs::SizePairOpt;
