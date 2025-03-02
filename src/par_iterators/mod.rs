#[cfg(test)]
mod tests;

mod par;
mod par_iter;
pub mod par_map;

pub use par::Par;
pub use par_iter::ParIter;
