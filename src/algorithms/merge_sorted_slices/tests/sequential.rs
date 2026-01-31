use super::utils::{SortKind, new_vec, split_to_sorted_vecs};
use crate::algorithms::data_structures::{Slice, SliceMut};
use crate::algorithms::merge_sorted_slices::alg::{MergeSortedSlicesParams, StreakSearch};
use crate::algorithms::merge_sorted_slices::sequential;
use crate::algorithms::merge_sorted_slices::tests::utils::SplitKind;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Debug;
use test_case::test_matrix;

fn elem_usize(x: usize) -> usize {
    x
}

fn elem_string(x: usize) -> String {
    x.to_string()
}

#[test_matrix(
    [elem_usize, elem_string],
    [0, 1, 2, 3, 37],
    [SortKind::Sorted, SortKind::ReverseSorted, SortKind::Mixed],
    [SplitKind::AllInLeft, SplitKind::AllInRight, SplitKind::OneInLeft, SplitKind::OneInRight, SplitKind::MoreInLeft, SplitKind::MoreInRight, SplitKind::Middle],
    [
        MergeSortedSlicesParams { num_threads: 1, streak_search: StreakSearch::None },
        MergeSortedSlicesParams { num_threads: 1, streak_search: StreakSearch::Linear },
        MergeSortedSlicesParams { num_threads: 1, streak_search: StreakSearch::Binary },
    ]
)]
fn merge_sorted_slices_seq<T: Ord + Clone + Debug>(
    elem: impl Fn(usize) -> T,
    len: usize,
    sort_kind: SortKind,
    split_kind: SplitKind,
    params: MergeSortedSlicesParams,
) {
    let input = new_vec(len, elem, sort_kind);
    let (mut left, mut right) = split_to_sorted_vecs(&input, split_kind);

    let mut result = Vec::with_capacity(input.len());
    sequential::merge_sorted_slices(
        |a, b| a < b,
        &Slice::from(left.as_slice()),
        &Slice::from(right.as_slice()),
        &mut SliceMut::from(&mut result),
        params,
    );

    // all elements of left & right are moved to result
    unsafe {
        result.set_len(left.len() + right.len());
        left.set_len(0);
        right.set_len(0);
    }

    let mut expected = input.clone();
    expected.sort();

    assert_eq!(result, expected);
}
