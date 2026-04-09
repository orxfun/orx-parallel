use crate::infallible_use::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use std::string::String;

const N: usize = 157;

#[test]
fn id_find() {
    let inputs = inputs(N);
    let result = inputs.into_par().first();
    assert_eq!(result, Some(String::from("0")));
}

#[test]
fn id_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn id_reduce() {
    let inputs = inputs(N);
    let result = inputs.into_par().reduce(|a, b| match a < b {
        true => b,
        false => a,
    });
    assert_eq!(result, Some(String::from("99")));
}
