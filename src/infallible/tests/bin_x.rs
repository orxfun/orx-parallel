use crate::collectables::par_col_into_test::ParCollectIntoTest;
use crate::infallible::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn bin_x_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter(|x| x.len() < 4)
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .first();
    assert_eq!(result, Some(0));
}

#[test]
fn bin_x_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter(|x| x.len() < 4)
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn bin_x_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter(|x| x.len() < 4)
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(160));
}

#[test_matrix(
    [Vec::new(), SplitVec::with_doubling_growth(), SplitVec::with_linear_growth(6), FixedVec::new(40)],
    [false, true]
)]
fn bin_x_collect_into<C: ParCollectIntoTest<String>>(c: C, pre_fill: bool) {
    let c = c.prepare(pre_fill, N / 5, |i| (1000 + i).to_string());

    let expected = c.expected(inputs(N).into_iter().filter(|x| x.len() < 4).flat_map(|x| {
        let a = x.parse::<u64>().unwrap();
        (0..5).map(move |i| (a + i).to_string())
    }));

    let result = inputs(N)
        .into_par()
        .filter(|x| x.len() < 4)
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .collect_into(c);

    assert_eq!(result, expected);
}
