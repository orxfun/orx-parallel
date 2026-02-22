use super::inputs::{SortKind, sorted_slices};
use crate::experiment::algorithms::merge_sorted_slices::par::{
    ParamsParMergeSortedSlices, PivotSearch, par_merge,
};
use crate::experiment::algorithms::merge_sorted_slices::seq::{
    ParamsSeqMergeSortedSlices, StreakSearch,
};
use crate::experiment::data_structures::slice_dst::SliceDst;
use crate::experiment::data_structures::slice_src::SliceSrc;
use crate::{DefaultPool, DefaultRunner, RunnerWithPool};
use alloc::string::String;
use alloc::vec::Vec;
use test_case::test_matrix;

fn runner() -> RunnerWithPool<DefaultPool> {
    DefaultRunner::default()
}

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
    [PARAMS[0],PARAMS[1],PARAMS[2],PARAMS[3],PARAMS[4],PARAMS[5]],
    [PivotSearch::Linear, PivotSearch::Binary],
    [0, 1, 64])
]
fn merge_sorted_slices_par(
    (left_len, total_len): (usize, usize),
    sort: SortKind,
    seq_params: ParamsSeqMergeSortedSlices,
    pivot_search: PivotSearch,
    chunk_size: usize,
) {
    let params = ParamsParMergeSortedSlices {
        seq_params: seq_params,
        chunk_size,
        num_threads: 4,
        pivot_search,
        put_large_to_left: seq_params.put_large_to_left,
    };

    run((left_len, total_len), sort, params);
}

#[test_matrix(
    [(0, 0), (0, 5), (5, 5), (4, 20), (10, 20), (14, 20)])
]
fn merge_sorted_slices_par_single_thread((left_len, total_len): (usize, usize)) {
    let params = ParamsParMergeSortedSlices {
        seq_params: PARAMS[5],
        chunk_size: 1,
        num_threads: 1,
        pivot_search: PivotSearch::Binary,
        put_large_to_left: PARAMS[5].put_large_to_left,
    };

    run((left_len, total_len), SortKind::Mixed, params);
}

#[cfg(not(miri))]
#[test_matrix(
    [1<<15],
    [0, 1, 64],
    [4, 16]
)]
fn merge_sorted_slices_par_large(len: usize, num_threads: usize, chunk_size: usize) {
    let params = ParamsParMergeSortedSlices {
        seq_params: ParamsSeqMergeSortedSlices {
            streak_search: StreakSearch::Linear,
            put_large_to_left: true,
        },
        pivot_search: PivotSearch::Binary,
        put_large_to_left: true,
        chunk_size,
        num_threads,
    };

    run((len / 2, len), SortKind::Mixed, params);
}

fn run((left_len, total_len): (usize, usize), sort: SortKind, params: ParamsParMergeSortedSlices) {
    let (mut expected, mut left, mut right) = sorted_slices(left_len, total_len, sort);

    let mut result = Vec::<String>::with_capacity(total_len);

    par_merge(
        |a, b| a < b,
        SliceSrc::from_slice(left.as_slice()),
        SliceSrc::from_slice(right.as_slice()),
        SliceDst::from_vec(&mut result),
        &params,
        runner(),
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
