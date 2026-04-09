use crate::infallible_use::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use std::string::String;

const N: usize = 157;

#[test]
fn bin_f_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using_clone(42)
        .filter(|u, x| {
            *u += 1;
            x.len() > 1
        })
        .filter(|u, x| {
            *u *= 2;
            x.len() < 4
        })
        .first();
    assert_eq!(result, Some(String::from("10")));
}

#[test]
fn bin_f_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using_clone(42)
        .filter(|u, x| {
            *u += 1;
            x.len() > 1
        })
        .filter(|u, x| {
            *u *= 2;
            x.len() < 4
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn bin_f_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using_clone(42)
        .filter(|u, x| {
            *u += 1;
            x.len() > 1
        })
        .filter(|u, x| {
            *u *= 2;
            x.len() < 4
        })
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(String::from("99")));
}
