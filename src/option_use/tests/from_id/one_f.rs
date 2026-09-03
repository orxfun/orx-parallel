use crate::option_use::tests::utils::{UseValue, inputs_opt};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn one_f_find_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .first();
    assert_eq!(result, Some(Some(String::from("10"))));
}

#[test]
fn one_f_find_any_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_f_reduce_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
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
fn one_f_reduce_err() {
    let inputs = inputs_opt(N, Some(42));
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
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

#[test]
fn one_f_collect_ok() {
    let result: Option<Vec<_>> = inputs_opt(N, None)
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .collect();
    assert!(result.is_some());
}

#[test]
fn one_f_collect_err() {
    let result: Option<Vec<_>> = inputs_opt(N, Some(42))
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .collect();
    assert!(result.is_none());
}
