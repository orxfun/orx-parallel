use crate::parameters::IterationOrder;
use crate::result_depr2::tests::utils::inputs;
use crate::*;
use std::string::String;
use std::vec;

const N: usize = 157;

#[test]
fn one_f_find_ok() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .filter(|x| x.len() > 1)
        .first();
    assert_eq!(result, Ok(Some(String::from("10"))));
}

#[test]
fn one_f_find_any_ok() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .filter(|x| x.len() > 1)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn one_f_reduce_ok() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .filter(|x| x.len() > 1)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Ok(Some(String::from("99"))));
}

#[test]
fn one_f_reduce_err() {
    let inputs = inputs(N, Some(42));
    let result = inputs
        .into_par()
        .fallible_result()
        .filter(|x| x.len() > 1)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Err(vec!['a']));
}
