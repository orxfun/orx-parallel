use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::parameters::IterationOrder;
use crate::result::tests::utils::inputs;
use crate::*;
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn id_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(Ok)
        .into_fallible()
        .first();
    assert_eq!(result, Ok(Some(String::from("0"))));
}

#[test]
fn id_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(Ok)
        .into_fallible()
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn id_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(Ok)
        .into_fallible()
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Ok(Some(String::from("99"))));
}

#[test]
fn id_reduce_ok_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| match x.as_str() == "42" {
            true => Ok(x),
            false => Err(vec!['a']),
        })
        .into_fallible()
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Err(vec!['a']));
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn id_collect_ok<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i.to_string(),
        inputs(N)
            .into_iter()
            .map::<Result<_, Vec<char>>, _>(Ok)
            .map(|x| x.unwrap())
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => inputs(N)
            .into_par()
            .map::<Result<_, Vec<char>>, _>(Ok)
            .into_fallible()
            .iteration_order(order)
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs(N)
            .into_par()
            .map::<Result<_, Vec<char>>, _>(Ok)
            .into_fallible()
            .iteration_order(order)
            .collect(),
    };

    C::assert_eq(result.unwrap(), expected, order);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn id_collect_err<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => inputs(N)
            .into_par()
            .map(|x| match x.as_str() == "42" {
                true => Ok(x),
                false => Err(vec!['a']),
            })
            .into_fallible()
            .iteration_order(order)
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs(N)
            .into_par()
            .map(|x| match x.as_str() == "42" {
                true => Ok(x),
                false => Err(vec!['a']),
            })
            .into_fallible()
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, Err(vec!['a']));
}
