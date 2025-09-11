#[cfg(test)]
mod tests;

pub mod fallible_option;
pub mod fallible_result;
mod map;
mod par;
mod xap;

pub use map::ParMap;
pub use par::Par;
pub use xap::ParXap;
