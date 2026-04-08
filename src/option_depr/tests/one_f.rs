use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use std::string::String;

const N: usize = 157;

#[test]
fn one_f_find_some() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() > 1)
        .first();
    assert_eq!(result, Some(Some(String::from("10"))));
}

#[test]
fn one_f_find_any_some() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() > 1)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_f_reduce_some() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() > 1)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(String::from("99"))));
}

#[test]
fn one_f_reduce_none() {
    let inputs = inputs(N, Some(42));
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() > 1)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}
