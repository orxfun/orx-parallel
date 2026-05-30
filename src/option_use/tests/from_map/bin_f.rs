use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::option_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn bin_f_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| Some(x))
        .into_optional()
        .using_clone(UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .first();
    assert_eq!(result, Some(Some(String::from("10"))));
}

#[test]
fn bin_f_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| Some(x))
        .into_optional()
        .using_clone(UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn bin_f_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using22(|th_idx| UseValue::new(th_idx))
        .map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
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
fn bin_f_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using22(|th_idx| UseValue::new(th_idx))
        .map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => Some(x),
                false => None,
            }
        })
        .into_optional()
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
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
fn bin_f_collect_ok<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i.to_string(),
        inputs(N)
            .into_iter()
            .map(|x| Some(x))
            .map(|x| x.unwrap())
            .filter(|x| x.len() > 1)
            .filter(|x| x.len() < 4)
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => inputs(N)
            .into_par()
            .using22(|th_idx| UseValue::new(th_idx))
            .map(|u, x| {
                u.mutate();
                Some(x)
            })
            .into_optional()
            .filter(|u, x| {
                u.mutate();
                x.len() > 1
            })
            .filter(|u, x| {
                u.mutate();
                x.len() < 4
            })
            .iteration_order(order)
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs(N)
            .into_par()
            .using22(|th_idx| UseValue::new(th_idx))
            .map(|u, x| {
                u.mutate();
                Some(x)
            })
            .into_optional()
            .filter(|u, x| {
                u.mutate();
                x.len() > 1
            })
            .filter(|u, x| {
                u.mutate();
                x.len() < 4
            })
            .iteration_order(order)
            .collect(),
    };

    C::assert_eq(result.unwrap(), expected, order);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn bin_f_collect_err<C: ParCollectIntoTest<String>>(
    _: C,
    mode: ColIntoMode,
    order: IterationOrder,
) {
    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => inputs(N)
            .into_par()
            .using22(|th_idx| UseValue::new(th_idx))
            .map(|u, x| {
                u.mutate();
                match x.as_str() == "42" {
                    true => Some(x),
                    false => None,
                }
            })
            .into_optional()
            .filter(|u, x| {
                u.mutate();
                x.len() > 1
            })
            .filter(|u, x| {
                u.mutate();
                x.len() < 4
            })
            .iteration_order(order)
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs(N)
            .into_par()
            .using22(|th_idx| UseValue::new(th_idx))
            .map(|u, x| {
                u.mutate();
                match x.as_str() == "42" {
                    true => Some(x),
                    false => None,
                }
            })
            .into_optional()
            .filter(|u, x| {
                u.mutate();
                x.len() > 1
            })
            .filter(|u, x| {
                u.mutate();
                x.len() < 4
            })
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, None);
}
