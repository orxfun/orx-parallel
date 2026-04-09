use crate::parameters::IterationOrder;
use crate::result_use::tests::utils::{UseValue, inputs_res};
use crate::*;
use std::string::String;
use std::vec;

const N: usize = 157;

#[test]
fn id_find_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs.into_par().fallible_result().first();
    assert_eq!(result, Ok(Some(String::from("0"))));
}

#[test]
fn id_find_any_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn id_reduce_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
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
        .fallible_result()
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Err(vec!['a']));
}
