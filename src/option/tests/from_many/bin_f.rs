use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use std::string::String;

const N: usize = 157;

#[test]
fn bin_f_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
        .fallible_option()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .first();
    assert_eq!(result, Some(Some(String::from("10"))));
}

#[test]
fn bin_f_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
        .fallible_option()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn bin_f_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
        .fallible_option()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(String::from("99"))));
}

#[test]
fn bin_f_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| match x.as_str() == "42" {
            true => [x.clone(), x.clone(), x].map(Some),
            false => [None, None, None],
        })
        .fallible_option()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}

#[test]
fn bin_f_collect_ok() {
    let inputs = inputs(N);
    let result: Option<std::vec::Vec<std::string::String>> = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
        .fallible_option()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .collect::<std::vec::Vec<_>>();
    assert!(result.is_some());
}

#[test]
fn bin_f_collect_err() {
    let inputs = inputs(N);
    let result: Option<std::vec::Vec<std::string::String>> = inputs
        .into_par()
        .flat_map(|x| match x.as_str() == "42" {
            true => [x.clone(), x.clone(), x].map(Some),
            false => [None, None, None],
        })
        .fallible_option()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .collect::<std::vec::Vec<_>>();
    assert_eq!(result, None);
}
