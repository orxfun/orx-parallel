use crate::infallible::tests::utils::inputs;
use crate::*;
use alloc::vec::Vec;
use std::string::{String, ToString};

const N: usize = 157;

#[test]
fn inf_first() {
    let input = inputs(N);

    let result = input.par().filter(|x| *x == &(N / 2).to_string()).first();
    assert_eq!(result, Some(&(N / 2).to_string()));

    let result = input.par().filter(|x| x.as_str() == "x").first();
    assert_eq!(result, None);

    // empty
    let input = inputs(0);
    let result = input.par().filter(|x| *x == &(N / 2).to_string()).first();
    assert_eq!(result, None);
}

#[test]
fn inf_reduce() {
    let input = inputs(N);

    let result = input.par().map(|x| x.len()).reduce(|a, b| a + b);
    assert_eq!(result, input.iter().map(|x| x.len()).reduce(|a, b| a + b));

    // empty
    let input = inputs(0);
    let result = input.par().map(|x| x.len()).reduce(|a, b| a + b);
    assert_eq!(result, None);
}

#[test]
fn inf_collect() {
    let input = inputs(N);

    let result: Vec<String> = input.into_par().filter(|x| x.len() < 2).collect();
}
