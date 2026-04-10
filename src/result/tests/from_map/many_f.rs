use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::parameters::IterationOrder;
use crate::result::tests::utils::inputs;
use crate::*;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn many_f_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .first();
    assert_eq!(result, Ok(Some(String::from("0"))));
}

#[test]
fn many_f_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn many_f_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Ok(Some(String::from("99"))));
}

#[test]
fn many_f_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| match x.as_str() == "42" {
            true => Ok(x),
            false => Err(vec!['a']),
        })
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
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
fn many_f_collect_ok<C: ParCollectIntoTest<String>>(
    _: C,
    mode: ColIntoMode,
    order: IterationOrder,
) {
    let expected = C::expected(
        mode,
        |i| i.to_string(),
        inputs(N)
            .into_par()
            .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
            .fallible_result()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect::<std::vec::Vec<_>>()
            .unwrap(),
    );

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(c) => inputs(N)
            .into_par()
            .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
            .fallible_result()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
            .fallible_result()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
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
fn many_f_collect_err<C: ParCollectIntoTest<String>>(
    _: C,
    mode: ColIntoMode,
    order: IterationOrder,
) {
    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(c) => inputs(N)
            .into_par()
            .map(|x| match x.as_str() == "42" {
                true => Ok(x),
                false => Err(vec!['a']),
            })
            .fallible_result()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .map(|x| match x.as_str() == "42" {
                true => Ok(x),
                false => Err(vec!['a']),
            })
            .fallible_result()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, Err(vec!['a']));
}
