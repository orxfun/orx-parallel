use crate::option::tests::utils::inputs_opt;
use crate::parameters::IterationOrder;
use crate::*;
use std::string::String;

const N: usize = 157;

#[test]
fn bin_f_find_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .first();
    assert_eq!(result, Some(Some(String::from("10"))));
}

#[test]
fn bin_f_find_any_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn bin_f_reduce_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(String::from("99"))));
}

#[test]
fn bin_f_reduce_err() {
    let inputs = inputs_opt(N, Some(42));
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}
