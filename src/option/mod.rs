#[cfg(test)]
mod tests;

mod par_iter;
mod par_runner;
pub mod size_pairs;
mod thread_execution;

pub use par_iter::ParOpt;
