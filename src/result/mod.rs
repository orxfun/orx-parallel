#[cfg(test)]
mod tests;

mod par_iter;
mod par_runner;
pub mod size_pairs;
mod thread_execution;
mod xap_res;

pub use par_iter::ParRes;
pub use xap_res::XapRes;
