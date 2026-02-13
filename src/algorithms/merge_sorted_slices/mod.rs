#[cfg(test)]
mod tests;

mod alg;
mod parallel;
mod sequential_test;

pub use alg::{MergeSortedSlicesParams, PivotSearch, StreakSearch, merge_sorted_slices};
