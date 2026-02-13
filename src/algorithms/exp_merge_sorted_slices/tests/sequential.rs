use crate::algorithms::data_structures::Slice;
use crate::algorithms::exp_merge_sorted_slices::sequential::{seq_merge, seq_merge_unchecked};
use crate::algorithms::exp_merge_sorted_slices::tests::inputs::{SortKind, sorted_slices};
use crate::algorithms::exp_merge_sorted_slices::{ParamsSeqMergeSortedSlices, StreakSearch};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;
use std::println;
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
        Slice::from(left.as_slice()),
        Slice::from(right.as_slice()),
        Slice::for_entire_capacity(&result),
        params,
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

#[test]
fn abc() {
    let left_len = 2;
    let total_len = 3;
    let sort = SortKind::Sorted;
    let params = PARAMS[0];

    let (mut expected, mut left, mut right) = sorted_slices(left_len, total_len, sort);

    let mut result = Vec::<String>::with_capacity(total_len);

    // let dst = result.as_ptr() as *mut String;
    // let source = right.as_ptr();
    // unsafe { dst.copy_from_nonoverlapping(source, result.capacity()) };

    // println!("src={}\ndst={}\n", source as usize, dst as usize);

    seq_merge(
        |a, b| a < b,
        Slice::from(left.as_slice()),
        Slice::from(right.as_slice()),
        Slice::for_entire_capacity(&result),
        params,
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
