use crate::{
    ParThreadPool, StdDefaultPool,
    sort::{slice::sort, tests::utils::create_input_and_sorted},
};
use test_case::test_matrix;

#[test_matrix(
    [0, 1, 1034],
    [0, 10000],
    [
        StdDefaultPool::default(),
        
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
