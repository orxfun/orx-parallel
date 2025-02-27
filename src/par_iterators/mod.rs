#[cfg(test)]
mod tests;

mod par;
mod par_iter;
mod par_map;

pub use par::Par;
pub use par_iter::ParIter;
pub use par_map::ParMap;
