use crate::option_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::String;

const N: usize = 157;

#[test]
fn id_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(Some)
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .first();
    assert_eq!(result, Some(Some(String::from("0"))));
}

#[test]
fn id_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(Some)
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn id_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
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
        .use_new(UseValue::new)
        .map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => Some(x),
                false => None,
            }
        })
        .into_optional()
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
fn id_collect_ok() {
    let result: Option<Vec<_>> = inputs(N)
        .into_par()
        .use_new(UseValue::new)
        .map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
        .collect();
    assert!(result.is_some());
}

#[test]
fn id_collect_err() {
    let result: Option<Vec<_>> = inputs(N)
        .into_par()
        .use_new(UseValue::new)
        .map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => Some(x),
                false => None,
            }
        })
        .into_optional()
        .collect();
    assert!(result.is_none());
}
