#[cfg(test)]
mod tests;

mod par;
mod par_core;
mod par_iter;
mod par_runner;
mod thread_execution;
mod xap_iter;

pub use par::ParResult;
pub use par_iter::ParResultIter;
pub use xap_iter::XapResultIter;
