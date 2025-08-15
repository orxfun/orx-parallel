#[cfg(test)]
mod tests;

mod map;
mod par;
pub mod result;
mod xap;

pub use map::ParMap;
pub use par::Par;
pub use xap::ParXap;
