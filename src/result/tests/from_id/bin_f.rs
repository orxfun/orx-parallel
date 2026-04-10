use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::parameters::IterationOrder;
use crate::result::tests::utils::inputs_res;
use crate::*;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn bin_f_find_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .first();
    assert_eq!(result, Ok(Some(String::from("10"))));
}

#[test]
fn bin_f_find_any_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn bin_f_reduce_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Ok(Some(String::from("99"))));
}

#[test]
fn bin_f_reduce_err() {
    let inputs = inputs_res(N, Some(42));
    let result = inputs
        .into_par()
        .fallible_result()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Err(vec!['a']));
}

#[test_matrix(
    [Vec::new(), SplitVec::with_doubling_growth(), SplitVec::with_linear_growth(6), FixedVec::new(40)],
    [ColIntoMode::Col, ColIntoMode::ColIntoEmpty, ColIntoMode::ColIntoFilled(N / 5)],
    [IterationOrder::Ordered, IterationOrder::Arbitrary]
)]
fn bin_f_collect_ok<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i.to_string(),
        inputs_res(N, None)
            .into_par()
            .fallible_result()
            .filter(|x| x.len() > 1)
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect::<std::vec::Vec<_>>()
            .unwrap(),
    );

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(c) => inputs_res(N, None)
            .into_par()
            .fallible_result()
            .filter(|x| x.len() > 1)
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect_into(c),
        None => inputs_res(N, None)
            .into_par()
            .fallible_result()
            .filter(|x| x.len() > 1)
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect(),
    };

    C::assert_eq(result.unwrap(), expected, order);
}

#[test_matrix(
[Vec::new(), SplitVec::with_doubling_growth(), SplitVec::with_linear_growth(6), FixedVec::new(40)],
[ColIntoMode::Col, ColIntoMode::ColIntoEmpty, ColIntoMode::ColIntoFilled(N / 5)],
[IterationOrder::Ordered, IterationOrder::Arbitrary]
)]
fn bin_f_collect_err<C: ParCollectIntoTest<String>>(
    _: C,
    mode: ColIntoMode,
    order: IterationOrder,
) {
    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(c) => inputs_res(N, Some(42))
            .into_par()
            .fallible_result()
            .filter(|x| x.len() > 1)
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect_into(c),
        None => inputs_res(N, Some(42))
            .into_par()
            .fallible_result()
            .filter(|x| x.len() > 1)
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, Err(vec!['a']));
}
