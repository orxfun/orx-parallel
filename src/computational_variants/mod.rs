#[cfg(test)]
mod tests;

pub mod fallible;
mod map;
pub mod optional;
pub mod optional_depr;
mod par;
mod xap;

pub use map::ParMap;
pub use par::Par;
pub use xap::ParXap;
