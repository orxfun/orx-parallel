use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::option::tests::utils::inputs_opt;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn id_find_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs.into_par().into_optional().first();
    assert_eq!(result, Some(Some(String::from("0"))));
}

#[test]
fn id_find_any_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn id_reduce_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(String::from("99"))));
}

#[test]
fn id_reduce_ok_err() {
    let inputs = inputs_opt(N, Some(42));
    let result = inputs
        .into_par()
        .into_optional()
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn id_collect_ok<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i.to_string(),
        inputs_opt(N, None)
            .into_iter()
            .map(|x| x.unwrap())
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(c) => inputs_opt(N, None)
            .into_par()
            .into_optional()
            .iteration_order(order)
            .collect_into(c),
        None => inputs_opt(N, None)
            .into_par()
            .into_optional()
            .iteration_order(order)
            .collect(),
    };

    C::assert_eq(result.unwrap(), expected, order);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn id_collect_err<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(c) => inputs_opt(N, Some(42))
            .into_par()
            .into_optional()
            .iteration_order(order)
            .collect_into(c),
        None => inputs_opt(N, Some(42))
            .into_par()
            .into_optional()
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, None);
}
