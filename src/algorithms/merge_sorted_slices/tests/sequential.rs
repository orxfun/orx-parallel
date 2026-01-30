use super::utils::{SortKind, new_vec, split_to_sorted_vecs};
use crate::algorithms::data_structures::{Slice, SliceMut};
use crate::algorithms::merge_sorted_slices::sequential;
use crate::algorithms::merge_sorted_slices::tests::utils::SplitKind;
use alloc::boxed::Box;
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

fn elem_box(x: usize) -> Box<usize> {
    Box::new(x)
}

#[test_matrix(
    [elem_usize, elem_string, elem_box],
    [0, 1, 2, 3, 37],
    [SortKind::Sorted, SortKind::ReverseSorted, SortKind::Mixed],
    [SplitKind::AllInLeft, SplitKind::AllInRight, SplitKind::OneInLeft, SplitKind::OneInRight, SplitKind::MoreInLeft, SplitKind::MoreInRight, SplitKind::Middle]
)]
fn merge_sorted_slices_seq<T: Ord + Clone + Debug>(
    elem: impl Fn(usize) -> T,
    len: usize,
    sort_kind: SortKind,
    split_kind: SplitKind,
) {
    let input = new_vec(len, elem, sort_kind);
    let (left, right) = split_to_sorted_vecs(&input, split_kind);

    let mut result = Vec::with_capacity(input.len());
    sequential::merge_sorted_slices(
        |a, b| a < b,
        &Slice::from(left.as_slice()),
        &Slice::from(right.as_slice()),
        &mut SliceMut::from(&mut result),
    );

    // all elements of left & right are moved to result
    unsafe { result.set_len(left.len() + right.len()) };
    core::mem::forget(left);
    core::mem::forget(right);

    let mut expected = input.clone();
    expected.sort();

    assert_eq!(result, expected);
}
