#[cfg(test)]
mod tests;

mod par_iter;
mod par_runner;
mod size_pairs;
mod thread_execution;

pub use par_iter::ParRes;
pub use size_pairs::SizePairRes;
