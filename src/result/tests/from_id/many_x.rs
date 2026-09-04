use crate::parameters::IterationOrder;
use crate::result::tests::utils::inputs_res;
use crate::*;
use std::format;
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

const N: usize = 157;

#[test]
fn many_x_find_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .into_fallible()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .first();
    assert_eq!(result, Ok(Some(String::from("0!"))));
}

#[test]
fn many_x_find_any_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .into_fallible()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn many_x_reduce_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .into_fallible()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Ok(Some(String::from("99!"))));
}

#[test]
fn many_x_reduce_err() {
    let inputs = inputs_res(N, Some(42));
    let result = inputs
        .into_par()
        .into_fallible()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Err(vec!['a']));
}

#[test]
fn many_x_collect_ok() {
    let result: Result<Vec<_>, Vec<char>> = inputs_res(N, None)
        .into_par()
        .into_fallible()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .collect();
    assert!(result.is_ok());
}

#[test]
fn many_x_collect_err() {
    let result: Result<Vec<_>, Vec<char>> = inputs_res(N, Some(42))
        .into_par()
        .into_fallible()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .collect();
    assert!(result.is_err());
}
