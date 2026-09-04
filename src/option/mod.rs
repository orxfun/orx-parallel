#[cfg(test)]
mod tests;

mod par;
mod par_core;
mod par_iter;
mod par_runner;
mod thread_execution;
mod xap_iter;

pub use par::ParOption;
pub use par_iter::ParOptionIter;
pub use xap_iter::XapOptionIter;
