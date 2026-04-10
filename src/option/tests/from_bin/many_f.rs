use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use std::string::{String, ToString};




const N: usize = 157;

#[test]
fn many_f_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .fallible_option()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .first();
    assert_eq!(result, Some(Some(String::from("0"))));
}

#[test]
fn many_f_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .fallible_option()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn many_f_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .fallible_option()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(String::from("99"))));
}

#[test]
fn many_f_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(match x.as_str() == "42" {
                true => Some(x),
                false => None,
            }),
        })
        .fallible_option()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}
