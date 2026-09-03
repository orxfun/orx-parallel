use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn one_x_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(Some)
        .into_optional()
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
        .map(Some)
        .into_optional()
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
        .map(Some)
        .into_optional()
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
        .map(|x| match x.as_str() == "42" {
            true => Some(x),
            false => None,
        })
        .into_optional()
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

#[test]
fn one_x_fold_ok() {
    let inputs = inputs(N);
    let mut expected = inputs
        .clone()
        .into_par()
        .map(Some)
        .into_optional()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .collect::<std::vec::Vec<_>>()
        .unwrap();
    expected.sort_unstable();

    let result = inputs
        .into_par()
        .map(Some)
        .into_optional()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .num_threads(4)
        .fold(Vec::new, |v, x| v.push(x));
    let result = result.unwrap();
    assert!(result.len() <= 4);
    let result = result
        .into_iter()
        .reduce(|mut a: Vec<u64>, mut b: Vec<u64>| {
            a.append(&mut b);
            a
        })
        .unwrap();
    let mut result = result;
    result.sort_unstable();

    assert_eq!(&result, &expected);
}

#[test]
fn one_x_fold_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| match x.as_str() == "42" {
            true => Some(x),
            false => None,
        })
        .into_optional()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .num_threads(4)
        .fold(Vec::new, |v, x| v.push(x));
    assert_eq!(result, None);
}

#[test]
fn one_x_collect_ok() {
    let result: Option<Vec<_>> = inputs(N)
        .into_par()
        .map(Some)
        .into_optional()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .collect();
    assert!(result.is_some());
}

#[test]
fn one_x_collect_err() {
    let result: Option<Vec<_>> = inputs(N)
        .into_par()
        .map(|x| match x.as_str() == "42" {
            true => Some(x),
            false => None,
        })
        .into_optional()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .collect();
    assert!(result.is_none());
}
