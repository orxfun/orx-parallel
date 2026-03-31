#[cfg(test)]
mod tests;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with None.
pub mod fallible_option;
/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with an error.
pub mod fallible_result;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with None.
pub mod fall_option;
/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with an error.
pub mod fall_result;

mod map;
mod par;
mod xap;
mod xap_fn;

pub use map::ParMap;
pub use par::Par;
pub use xap::ParXap;
