use crate::sort::merge_slices::{MergeSliceKind, merge_slices};
use crate::sort::slice_chunk::SliceChunk;
use crate::sort::tests::utils::create_input;
use alloc::boxed::Box;
use alloc::vec::Vec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use test_case::test_matrix;

#[test_matrix(
    [0, 1, 1034],
    [0, 10000],
    [
        MergeSliceKind::Sequential
    ]
)]
fn merge_ordered_slices(len: usize, number_of_swaps: usize, kind: MergeSliceKind) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut slice1 = create_input(
        len,
        |_| Box::new(rng.random_range(0..10 * len)),
        number_of_swaps,
    );
    slice1.sort();

    let mut slice2 = create_input(
        len,
        |_| Box::new(rng.random_range(0..10 * len)),
        number_of_swaps,
    );
    slice2.sort();

    let mut result: Vec<_> = slice1
        .iter()
        .cloned()
        .chain(slice2.iter().cloned())
        .collect();
    let mut expected = result.clone();
    expected.sort();

    let src1 = SliceChunk::from(slice1.as_mut_slice());
    let src2 = SliceChunk::from(slice2.as_mut_slice());
    let dst = SliceChunk::from(result.as_mut_slice());

    merge_slices(kind, src1, src2, dst);

    assert_eq!(expected, result);
}
