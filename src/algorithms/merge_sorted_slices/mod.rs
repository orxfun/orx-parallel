#[cfg(test)]
mod tests;

mod alg;
mod sequential;

pub use alg::{MergeSortedSlicesParams, PivotSearch, StreakSearch, merge_sorted_slices};
