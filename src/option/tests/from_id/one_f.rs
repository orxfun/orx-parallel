use crate::option::tests::utils::inputs_opt;
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
        .filter(|x| x.len() > 1)
        .first();
    assert_eq!(result, Some(Some(String::from("10"))));
}

#[test]
fn one_f_find_any_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
        .filter(|x| x.len() > 1)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_f_reduce_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
        .filter(|x| x.len() > 1)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(String::from("99"))));
}

#[test]
fn one_f_reduce_err() {
    let inputs = inputs_opt(N, Some(42));
    let result = inputs
        .into_par()
        .into_optional()
        .filter(|x| x.len() > 1)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}

#[test]
fn one_f_fold_ok() {
    let inputs = inputs_opt(N, None);
    let expected = inputs
        .clone()
        .into_par()
        .into_optional()
        .filter(|x| x.len() > 1)
        .collect::<std::vec::Vec<_>>()
        .unwrap();

    let mut expected_flat = String::new();
    expected.iter().for_each(|x| expected_flat.push_str(x));
    let mut expected: Vec<_> = expected_flat.chars().collect();
    expected.sort();

    let result = inputs
        .into_par()
        .into_optional()
        .filter(|x| x.len() > 1)
        .num_threads(4)
        .fold(String::new, |s, x| s.push_str(&x));
    let result = result.unwrap();
    assert!(result.len() <= 4);
    let result = result
        .into_iter()
        .reduce(|mut a: String, b: String| {
            a.push_str(&b);
            a
        })
        .unwrap();
    let mut result: Vec<_> = result.chars().collect();
    result.sort();

    assert_eq!(&result, &expected);
}

#[test]
fn one_f_fold_err() {
    let inputs = inputs_opt(N, Some(42));
    let result = inputs
        .into_par()
        .into_optional()
        .filter(|x| x.len() > 1)
        .num_threads(4)
        .fold(String::new, |s, x| s.push_str(&x));
    assert_eq!(result, None);
}

#[test]
fn one_f_collect_ok() {
    let result: Option<Vec<_>> = inputs_opt(N, None)
        .into_par()
        .into_optional()
        .filter(|x| x.len() > 1)
        .collect();
    assert!(result.is_some());
}

#[test]
fn one_f_collect_err() {
    let result: Option<Vec<_>> = inputs_opt(N, Some(42))
        .into_par()
        .into_optional()
        .filter(|x| x.len() > 1)
        .collect();
    assert!(result.is_none());
}
