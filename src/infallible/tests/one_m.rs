use crate::collectables::par_col_into_test::ParCollectIntoTest;
use crate::infallible::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::format;
use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn one_m_find() {
    let inputs = inputs(N);
    let result = inputs.into_par().map(|x| x.parse::<u64>().unwrap()).first();
    assert_eq!(result, Some(0));
}

#[test]
fn one_m_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| x.parse::<u64>().unwrap())
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_m_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(156));
}

#[test_matrix(
    [Vec::new(), SplitVec::with_doubling_growth(), SplitVec::with_linear_growth(6), FixedVec::new(40)],
    [false, true]
)]
fn one_m_collect_into<C: ParCollectIntoTest<String>>(c: C, pre_fill: bool) {
    let c = c.prepare(pre_fill, N / 5, |i| (1000 + i).to_string());

    let expected = c.expected(inputs(N).into_iter().map(|x| format!("{}0", x)));

    let result = inputs(N)
        .into_par()
        .map(|x| format!("{}0", x))
        .collect_into(c);

    assert_eq!(result, expected);
}
