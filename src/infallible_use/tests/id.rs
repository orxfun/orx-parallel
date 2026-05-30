use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::infallible_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn id_find() {
    let inputs = inputs(N);
    let result = inputs.into_par().using_clone(UseValue::new(42)).first();
    assert_eq!(result, Some(String::from("0")));
}

#[test]
fn id_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using22(|th_idx| UseValue::new(th_idx))
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn id_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using_clone(UseValue::new(42))
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, Some(String::from("99")));
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn id_collect<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let iter = || inputs(N).into_iter();

    let expected = C::expected(mode, |i| i.to_string(), iter());

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => {
            inputs(N)
                .into_par()
                .using22(|th_idx| UseValue::new(th_idx))
                .iteration_order(order)
                .collect_into(&mut c);
            c
        }
        None => inputs(N)
            .into_par()
            .using22(|th_idx| UseValue::new(th_idx))
            .iteration_order(order)
            .collect(),
    };

    C::assert_eq(result, expected, order);
}
