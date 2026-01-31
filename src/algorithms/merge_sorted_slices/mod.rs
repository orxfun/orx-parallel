#[cfg(test)]
mod tests;

mod alg;
mod sequential;

pub use alg::MergeSortedSlicesParams;
pub use alg::merge_sorted_slices;
