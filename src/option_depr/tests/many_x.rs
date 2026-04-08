use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use std::format;
use std::string::{String, ToString};

const N: usize = 157;

#[test]
fn many_x_find_some() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .first();
    assert_eq!(result, Some(Some(String::from("0!"))));
}

#[test]
fn many_x_find_any_some() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn many_x_reduce_some() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(String::from("99!"))));
}

#[test]
fn many_x_reduce_none() {
    let inputs = inputs(N, Some(42));
    let result = inputs
        .into_par()
        .fallible_option()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}
