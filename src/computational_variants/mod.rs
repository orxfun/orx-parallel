#[cfg(test)]
mod tests;

pub(crate) mod fallible_option;
mod fallible_result;
mod map;
mod par;
mod xap;

pub use map::ParMap;
pub use par::Par;
pub use xap::ParXap;
