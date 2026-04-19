#[cfg(test)]
mod tests;

mod par;
mod par_iter;
mod par_runner;
mod thread_execution;

pub use par::ParOpt;
pub use par_iter::ParOptIter;
pub use par_runner::ParRunnerOpt;
