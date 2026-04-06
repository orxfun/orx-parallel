use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use std::string::String;

const N: usize = 157;

#[test]
fn id_find_some() {
    let inputs = inputs(N, None);
    let result = inputs.into_par().fallible_option().first();
    assert_eq!(result, Some(Some(String::from("0"))));
}

#[test]
fn id_find_any_some() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn id_reduce_some() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(String::from("99"))));
}

#[test]
fn id_reduce_ok_none() {
    let inputs = inputs(N, Some(42));
    let result = inputs
        .into_par()
        .fallible_option()
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}
