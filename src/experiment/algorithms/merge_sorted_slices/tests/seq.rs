use super::inputs::{SortKind, sorted_slices};
use crate::experiment::algorithms::merge_sorted_slices::seq::{
    ParamsSeqMergeSortedSlices, StreakSearch, seq_merge,
};
use crate::experiment::data_structures::slice_dst::SliceDst;
use crate::experiment::data_structures::slice_src::SliceSrc;
use alloc::string::String;
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

#[test_matrix(
    [(0, 0), (0, 5), (5, 5), (4, 20), (10, 20), (14, 20)],
    [SortKind::Sorted, SortKind::ReverseSorted, SortKind::Mixed],
    [PARAMS[0],PARAMS[1],PARAMS[2],PARAMS[3],PARAMS[4],PARAMS[5]])
]
fn xyz((left_len, total_len): (usize, usize), sort: SortKind, params: ParamsSeqMergeSortedSlices) {
    let (mut expected, mut left, mut right) = sorted_slices(left_len, total_len, sort);

    let mut result = Vec::<String>::with_capacity(total_len);

    seq_merge(
        |a, b| a < b,
        SliceSrc::from_slice(left.as_slice()),
        SliceSrc::from_slice(right.as_slice()),
        SliceDst::from_vec(&mut result),
        &params,
    );

    // all elements of left & right are moved to result
    unsafe {
        result.set_len(left.len() + right.len());
        left.set_len(0);
        right.set_len(0);
    }

    expected.sort();
    assert_eq!(result, expected);
}
