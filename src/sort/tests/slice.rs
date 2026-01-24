use crate::{
    StdDefaultPool,
    sort::{slice::sort, tests::utils::create_input},
};
use std::string::ToString;
use test_case::test_matrix;

#[test_matrix(
    [0, 1, 1034],
    [0, 10000],
    [0, 1, 4]
)]
fn slice_sort(len: usize, number_of_swaps: usize, nt: usize) {
    let mut pool = StdDefaultPool::default();
    let mut v = create_input(len, |i| i.to_string(), number_of_swaps);
    sort(&mut pool, &mut v);
}
