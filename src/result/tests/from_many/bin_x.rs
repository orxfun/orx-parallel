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
fn bin_x_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok))
        .fallible_result()
        .filter(|x| x.len() < 4)
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .first();
    assert_eq!(result, Ok(Some(0)));
}

#[test]
fn bin_x_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok))
        .fallible_result()
        .filter(|x| x.len() < 4)
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn bin_x_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok))
        .fallible_result()
        .filter(|x| x.len() < 4)
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Ok(Some(160)));
}

#[test]
fn bin_x_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| match x.as_str() == "42" {
            true => [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok),
            false => [Err(vec!['a']), Err(vec!['b']), Err(vec!['c'])],
        })
        .fallible_result()
        .filter(|x| x.len() < 4)
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
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
fn bin_x_collect_ok<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i as u64,
        inputs(N)
            .into_par()
            .flat_map(|x| [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok))
            .fallible_result()
            .filter(|x| x.len() < 4)
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| a + i)
            })
            .iteration_order(order)
            .collect::<std::vec::Vec<_>>()
            .unwrap(),
    );

    let result = match C::init_result(mode, |i| i as u64) {
        Some(c) => inputs(N)
            .into_par()
            .flat_map(|x| [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok))
            .fallible_result()
            .filter(|x| x.len() < 4)
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| a + i)
            })
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .flat_map(|x| [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok))
            .fallible_result()
            .filter(|x| x.len() < 4)
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| a + i)
            })
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
fn bin_x_collect_err<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i as u64) {
        Some(c) => inputs(N)
            .into_par()
            .flat_map(|x| match x.as_str() == "42" {
                true => [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok),
                false => [Err(vec!['a']), Err(vec!['b']), Err(vec!['c'])],
            })
            .fallible_result()
            .filter(|x| x.len() < 4)
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| a + i)
            })
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .flat_map(|x| match x.as_str() == "42" {
                true => [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok),
                false => [Err(vec!['a']), Err(vec!['b']), Err(vec!['c'])],
            })
            .fallible_result()
            .filter(|x| x.len() < 4)
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| a + i)
            })
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, Err(vec!['a']));
}
