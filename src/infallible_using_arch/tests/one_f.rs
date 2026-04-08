use crate::infallible::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use std::string::String;

const N: usize = 157;

#[test]
fn one_f_find() {
    let inputs = inputs(N);
    let result = inputs.into_par().filter(|x| x.len() > 1).first();
    assert_eq!(result, Some(String::from("10")));
}

#[test]
fn one_f_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter(|x| x.len() > 1)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_f_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter(|x| x.len() > 1)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(String::from("99")));
}
