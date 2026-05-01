use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::infallible::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn bin_f_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .first();
    assert_eq!(result, Some(String::from("10")));
}

#[test]
fn bin_f_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn bin_f_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(String::from("99")));
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn bin_f_collect<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let iter = || {
        inputs(N)
            .into_iter()
            .filter(|x| x.len() > 1)
            .filter(|x| x.len() < 4)
    };

    let expected = C::expected(mode, |i| i.to_string(), iter());

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => {
            inputs(N)
                .into_par()
                .iteration_order(order)
                .filter(|x| x.len() > 1)
                .filter(|x| x.len() < 4)
                .collect_into(&mut c);
            c
        }
        None => inputs(N)
            .into_par()
            .iteration_order(order)
            .filter(|x| x.len() > 1)
            .filter(|x| x.len() < 4)
            .collect(),
    };

    C::assert_eq(result, expected, order);
}
