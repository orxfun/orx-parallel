use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn many_f_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(Some)
        .into_optional()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .first();
    assert_eq!(result, Some(Some(String::from("0"))));
}

#[test]
fn many_f_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(Some)
        .into_optional()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn many_f_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(Some)
        .into_optional()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(String::from("99"))));
}

#[test]
fn many_f_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| match x.as_str() == "42" {
            true => Some(x),
            false => None,
        })
        .into_optional()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn many_f_collect_ok<C: ParCollectIntoTest<String>>(
    _: C,
    mode: ColIntoMode,
    order: IterationOrder,
) {
    let expected = C::expected(
        mode,
        |i| i.to_string(),
        inputs(N)
            .into_iter()
            .map(Some)
            .map(|x| x.unwrap())
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|x| x.len() < 4)
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => inputs(N)
            .into_par()
            .map(Some)
            .into_optional()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs(N)
            .into_par()
            .map(Some)
            .into_optional()
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

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn many_f_collect_err<C: ParCollectIntoTest<String>>(
    _: C,
    mode: ColIntoMode,
    order: IterationOrder,
) {
    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => inputs(N)
            .into_par()
            .map(|x| match x.as_str() == "42" {
                true => Some(x),
                false => None,
            })
            .into_optional()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs(N)
            .into_par()
            .map(|x| match x.as_str() == "42" {
                true => Some(x),
                false => None,
            })
            .into_optional()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|x| x.len() < 4)
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, None);
}
