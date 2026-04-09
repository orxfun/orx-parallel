use crate::collectables::par_col_into_test::ParCollectIntoTest;
use crate::infallible::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use std::format;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn many_x_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .first();
    assert_eq!(result, Some(String::from("0!")));
}

#[test]
fn many_x_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn many_x_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(String::from("99!")));
}

#[test_matrix(
    [Vec::new(), SplitVec::with_doubling_growth(), SplitVec::with_linear_growth(6), FixedVec::new(40)],
    [false, true]
)]
fn many_x_collect_into<C: ParCollectIntoTest<String>>(c: C, pre_fill: bool) {
    let c = c.prepare(pre_fill, N / 5, |i| (1000 + i).to_string());

    let expected = c.expected(
        inputs(N)
            .into_iter()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .flat_map(|x| [format!("{x}!"), x]),
    );

    let result = inputs(N)
        .into_par()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .collect_into(c);

    assert_eq!(result, expected);
}
