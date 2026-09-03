use crate::parameters::IterationOrder;
use crate::result::tests::utils::inputs_res;
use crate::*;
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn id_find_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs.into_par().into_fallible().first();
    assert_eq!(result, Ok(Some(String::from("0"))));
}

#[test]
fn id_find_any_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .into_fallible()
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn id_reduce_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .into_fallible()
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Ok(Some(String::from("99"))));
}

#[test]
fn id_reduce_ok_err() {
    let inputs = inputs_res(N, Some(42));
    let result = inputs
        .into_par()
        .into_fallible()
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Err(vec!['a']));
}

#[test]
fn id_collect_ok() {
    let result: Result<Vec<_>, Vec<char>> =
        inputs_res(N, None).into_par().into_fallible().collect();
    assert!(result.is_ok());
}

#[test]
fn id_collect_err() {
    let result: Result<Vec<_>, Vec<char>> =
        inputs_res(N, Some(42)).into_par().into_fallible().collect();
    assert!(result.is_err());
}
