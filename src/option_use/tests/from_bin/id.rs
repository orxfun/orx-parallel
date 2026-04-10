use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::option_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn id_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .fallible_option()
        .using_clone(UseValue::new(42))
        .first();
    assert_eq!(result, Some(Some(String::from("0"))));
}

#[test]
fn id_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .fallible_option()
        .using_clone(UseValue::new(42))
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn id_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .filter_map(|u, x| {
            u.mutate();
            match x.as_str() == "7" {
                true => None,
                false => Some(Some(x)),
            }
        })
        .fallible_option()
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, Some(Some(String::from("99"))));
}

#[test]
fn id_reduce_ok_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .filter_map(|u, x| {
            u.mutate();
            match x.as_str() == "7" {
                true => None,
                false => Some(match x.as_str() == "42" {
                    true => Some(x),
                    false => None,
                }),
            }
        })
        .fallible_option()
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, None);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn id_collect_ok<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i.to_string(),
        inputs(N)
            .into_iter()
            .filter_map(|x| match x.as_str() == "7" {
                true => None,
                false => Some(Some(x)),
            })
            .map(|x| x.unwrap())
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(c) => inputs(N)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map(|u, x| {
                u.mutate();
                match x.as_str() == "7" {
                    true => None,
                    false => Some(Some(x)),
                }
            })
            .fallible_option()
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map(|u, x| {
                u.mutate();
                match x.as_str() == "7" {
                    true => None,
                    false => Some(Some(x)),
                }
            })
            .fallible_option()
            .iteration_order(order)
            .collect(),
    };

    C::assert_eq(result.unwrap(), expected, order);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn id_collect_err<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(c) => inputs(N)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map(|u, x| {
                u.mutate();
                match x.as_str() == "7" {
                    true => None,
                    false => Some(match x.as_str() == "42" {
                        true => Some(x),
                        false => None,
                    }),
                }
            })
            .fallible_option()
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map(|u, x| {
                u.mutate();
                match x.as_str() == "7" {
                    true => None,
                    false => Some(match x.as_str() == "42" {
                        true => Some(x),
                        false => None,
                    }),
                }
            })
            .fallible_option()
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, None);
}
