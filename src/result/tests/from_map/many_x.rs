use crate::parameters::IterationOrder;
use crate::result::tests::utils::inputs_map;
use crate::*;
use std::format;
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

const N: usize = 157;

#[test]
fn many_x_find_ok() {
    let inputs = inputs_map(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
        .fallible_result()
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
    let inputs = inputs_map(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
        .fallible_result()
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
    let inputs = inputs_map(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
        .fallible_result()
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
    let inputs = inputs_map(N);
    let result = inputs
        .into_par()
        .map(|x| match x.as_str() == "42" {
            true => Ok(x),
            false => Err(vec!['a']),
        })
        .fallible_result()
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
