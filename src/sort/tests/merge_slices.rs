use crate::sort::merge_slices::{MergeSliceKind, merge_slices};
use crate::sort::slice_chunks::Slice;
use alloc::boxed::Box;
use alloc::vec::Vec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use test_case::test_matrix;

#[test_matrix(
    [0, 1, 2, 3, 1000],
    [
        MergeSliceKind::Sequential
    ]
)]
fn merge_ordered_slices(len: usize, kind: MergeSliceKind) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut elem = || rng.random_range(0..10 * len);

    let mut input: Vec<_> = (0..len).map(|_| elem()).collect();
    let (slice1, slice2) = input.split_at_mut(len / 2);
    slice1.sort();
    slice2.sort();

    let mut expected = input.clone();
    expected.sort();

    let expected: Vec<_> = expected.into_iter().map(Box::new).collect();
    let mut input: Vec<_> = input.into_iter().map(Box::new).collect();

    let (slice1, slice2) = input.split_at_mut(len / 2);
    let mut result: Vec<_> = Vec::with_capacity(len);

    let src1 = Slice::from(slice1);
    let src2 = Slice::from(slice2);
    let dst = Slice::new(result.as_mut_ptr(), len);

    merge_slices(kind, src1, src2, dst);

    unsafe {
        result.set_len(len);
        input.set_len(0);
    }

    assert_eq!(expected, result);
}

#[test]
fn xyz() {
    let len = 4;
    let kind = MergeSliceKind::Sequential;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut elem = || rng.random_range(0..10 * len);

    let mut input: Vec<_> = (0..len).map(|_| elem()).collect();
    let (slice1, slice2) = input.split_at_mut(len / 2);
    slice1.sort();
    slice2.sort();

    let mut expected = input.clone();
    expected.sort();

    // let expected: Vec<_> = expected.into_iter().map(Box::new).collect();
    // let mut input: Vec<_> = input.into_iter().map(Box::new).collect();

    let expected: Vec<_> = expected.into_iter().map(|x| x).collect();
    let mut input: Vec<_> = input.into_iter().map(|x| x).collect();

    let (slice1, slice2) = input.split_at_mut(len / 2);
    let mut result: Vec<_> = Vec::with_capacity(len);

    let src1 = Slice::from(slice1);
    let src2 = Slice::from(slice2);
    let dst = Slice::new(result.as_mut_ptr(), len);

    merge_slices(kind, src1, src2, dst);

    unsafe {
        result.set_len(len);
        input.set_len(0);
    }

    std::dbg!(&input);
    std::dbg!(&result);

    assert_eq!(expected, result);
}
