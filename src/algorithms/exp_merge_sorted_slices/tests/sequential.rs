use crate::algorithms::exp_merge_sorted_slices::{ParamsSeqMergeSortedSlices, StreakSearch};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use test_case::test_matrix;

const PARAMS: &[ParamsSeqMergeSortedSlices] = &[
    ParamsSeqMergeSortedSlices {
        streak_search: StreakSearch::None,
        put_large_to_left: false,
    },
    ParamsSeqMergeSortedSlices {
        streak_search: StreakSearch::Linear,
        put_large_to_left: false,
    },
    ParamsSeqMergeSortedSlices {
        streak_search: StreakSearch::Binary,
        put_large_to_left: false,
    },
    ParamsSeqMergeSortedSlices {
        streak_search: StreakSearch::None,
        put_large_to_left: true,
    },
    ParamsSeqMergeSortedSlices {
        streak_search: StreakSearch::Linear,
        put_large_to_left: true,
    },
    ParamsSeqMergeSortedSlices {
        streak_search: StreakSearch::Binary,
        put_large_to_left: true,
    },
];

fn elem_usize(x: usize) -> usize {
    x
}

fn elem_string(x: usize) -> String {
    x.to_string()
}
