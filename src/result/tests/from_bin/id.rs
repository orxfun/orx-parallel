use crate::parameters::IterationOrder;
use crate::result::tests::utils::inputs;
use crate::*;
use std::string::String;
use std::vec;
use std::vec::Vec;

const N: usize = 157;

#[test]
fn id_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map::<Result<_, Vec<char>>, _>(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Ok(x)),
        })
        .fallible_result()
        .first();
    assert_eq!(result, Ok(Some(String::from("0"))));
}

#[test]
fn id_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map::<Result<_, Vec<char>>, _>(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Ok(x)),
        })
        .fallible_result()
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn id_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map::<Result<_, Vec<char>>, _>(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Ok(x)),
        })
        .fallible_result()
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Ok(Some(String::from("99"))));
}

#[test]
fn id_reduce_ok_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map::<Result<_, Vec<char>>, _>(|x| match x.as_str() == "7" {
            true => None,
            false => Some(match x.as_str() == "42" {
                true => Ok(x),
                false => Err(vec!['a']),
            }),
        })
        .fallible_result()
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Err(vec!['a']));
}
