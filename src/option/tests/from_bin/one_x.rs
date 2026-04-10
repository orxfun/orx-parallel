use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;

const N: usize = 157;

#[test]
fn one_x_find_ok() {
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
            (0..5).map(move |i| a + i)
        })
        .first();
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn one_x_find_any_ok() {
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
            (0..5).map(move |i| a + i)
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_x_reduce_ok() {
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
            (0..5).map(move |i| a + i)
        })
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(160)));
}

#[test]
fn one_x_reduce_err() {
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
            (0..5).map(move |i| a + i)
        })
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}
