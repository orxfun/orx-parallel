use crate::sort::{slice::sort, tests::utils::create_input_and_sorted};
use crate::{ParThreadPool, StdDefaultPool};
use alloc::vec;
use alloc::vec::Vec;
use core::num::NonZeroUsize;
use core::ops::Range;
use test_case::{test_case, test_matrix};

#[test_case(0, 0, vec![])]
#[test_case(0, 1, vec![])]
#[test_case(10, 0, vec![])]
#[test_case(15, 5, vec![0..3, 3..6, 6..9, 9..12, 12..15])]
#[test_case(15, 4, vec![0..4, 4..8, 8..12, 12..15])]
#[test_case(15, 2, vec![0..8, 8..15])]
#[test_case(15, 1, vec![0..15])]
#[test_case(15, 6, vec![0..3, 3..6, 6..9, 9..11, 11..13, 13..15])]
#[test_case(15, 10, vec![0..2, 2..4, 4..6, 6..8, 8..10, 10..11, 11..12, 12..13, 13..14, 14..15])]
fn slice_ranges(len: usize, num_chunks: usize, ranges: Vec<Range<usize>>) {
    assert_eq!(crate::sort::slice::slice_ranges(len, num_chunks), ranges);
}

#[test_matrix(
    [0, 1, 1034],
    [0, 10000],
    [
        StdDefaultPool::default(),
        StdDefaultPool::with_max_num_threads(NonZeroUsize::new(1).unwrap()),
        StdDefaultPool::with_max_num_threads(NonZeroUsize::new(4).unwrap()),
    ]
)]
fn slice_sort<P>(len: usize, number_of_swaps: usize, mut pool: P)
where
    P: ParThreadPool,
{
    let (mut input, sorted) = create_input_and_sorted(len, |i| i, number_of_swaps);
    sort(&mut pool, &mut input);
    assert_eq!(input, sorted);
}
