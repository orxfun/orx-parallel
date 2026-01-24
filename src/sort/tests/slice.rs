use crate::sort::{slice::sort, tests::utils::create_input_and_sorted};
use crate::{ParThreadPool, StdDefaultPool};
use core::num::NonZeroUsize;
use test_case::test_matrix;

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
