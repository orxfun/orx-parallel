use crate::sort::merge_slices::{MergeSliceKind, merge_slices};
use crate::sort::slice_chunk::SliceChunk;
use alloc::boxed::Box;
use alloc::vec::Vec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use test_case::test_matrix;

#[test_matrix(
    [1000],
    [
        MergeSliceKind::Sequential
    ]
)]
fn merge_ordered_slices(len: usize, kind: MergeSliceKind) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut elem = || Box::new(rng.random_range(0..10 * len));

    let mut input: Vec<_> = (0..len).map(|_| elem()).collect();
    let mut expected = input.clone();
    expected.sort();

    let (slice1, slice2) = input.split_at_mut(len / 2);
    slice1.sort();
    slice2.sort();

    let mut result: Vec<_> = Vec::with_capacity(len);

    let src1 = SliceChunk::from(slice1);
    let src2 = SliceChunk::from(slice2);
    let dst = SliceChunk::new(result.as_mut_ptr(), len);

    merge_slices(kind, src1, src2, dst);

    unsafe {
        result.set_len(len);
        input.set_len(0);
    }

    assert_eq!(expected, result);
}
