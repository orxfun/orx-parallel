use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::parameters::IterationOrder;
use crate::result::tests::utils::inputs;
use crate::*;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use std::vec;
use std::vec::Vec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn bin_m_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
        .fallible_result()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .first();
    assert_eq!(result, Ok(Some(0)));
}

#[test]
fn bin_m_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
        .fallible_result()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn bin_m_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
        .fallible_result()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Ok(Some(156)));
}

#[test]
fn bin_m_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| match x.as_str() == "42" {
            true => Ok(x),
            false => Err(vec!['a']),
        })
        .fallible_result()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Err(vec!['a']));
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn bin_m_collect_ok<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i as u64,
        inputs(N)
            .into_iter()
            .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
            .map(|x| x.unwrap())
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i as u64) {
        Some(c) => inputs(N)
            .into_par()
            .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
            .fallible_result()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
            .fallible_result()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect(),
    };

    C::assert_eq(result.unwrap(), expected, order);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn bin_m_collect_err<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i as u64) {
        Some(c) => inputs(N)
            .into_par()
            .map(|x| match x.as_str() == "42" {
                true => Ok(x),
                false => Err(vec!['a']),
            })
            .fallible_result()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .map(|x| match x.as_str() == "42" {
                true => Ok(x),
                false => Err(vec!['a']),
            })
            .fallible_result()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, Err(vec!['a']));
}
