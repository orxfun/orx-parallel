#[cfg(test)]
mod tests;

mod params;
mod sequential;

pub use params::{
    ParamsParMergeSortedSlices, ParamsSeqMergeSortedSlices, SplitPivotSearch, StreakSearch,
};
