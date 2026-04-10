use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn bin_m_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
        .fallible_option()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .first();
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn bin_m_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
        .fallible_option()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn bin_m_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
        .fallible_option()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(156)));
}

#[test]
fn bin_m_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| match x.as_str() == "42" {
            true => [x.clone(), x.clone(), x].map(Some),
            false => [None, None, None],
        })
        .fallible_option()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}

#[test_matrix(
    [Vec::new(), SplitVec::with_doubling_growth(), SplitVec::with_linear_growth(6), FixedVec::new(40)],
    [ColIntoMode::Col, ColIntoMode::ColIntoEmpty, ColIntoMode::ColIntoFilled(N / 5)],
    [IterationOrder::Ordered, IterationOrder::Arbitrary]
)]
fn bin_m_collect_ok<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i as u64,
        inputs(N)
            .into_iter()
            .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
            .map(|x| x.unwrap())
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i as u64) {
        Some(c) => inputs(N)
            .into_par()
            .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
            .fallible_option()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
            .fallible_option()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
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
fn bin_m_collect_err<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i as u64) {
        Some(c) => inputs(N)
            .into_par()
            .flat_map(|x| match x.as_str() == "42" {
                true => [x.clone(), x.clone(), x].map(Some),
                false => [None, None, None],
            })
            .fallible_option()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .flat_map(|x| match x.as_str() == "42" {
                true => [x.clone(), x.clone(), x].map(Some),
                false => [None, None, None],
            })
            .fallible_option()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, None);
}
