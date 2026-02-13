#[cfg(test)]
mod tests;

mod exp_alg;
mod exp_sequential;
mod parallel;
mod params;
mod sequential;

pub use exp_alg::{ExpMergeSortedSlicesParams, PivotSearch, StreakSearch, merge_sorted_slices};
