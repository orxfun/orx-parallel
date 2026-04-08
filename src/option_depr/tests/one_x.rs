use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;

const N: usize = 157;

#[test]
fn one_x_find_some() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .first();
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn one_x_find_any_some() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
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
fn one_x_reduce_some() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
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
fn one_x_reduce_none() {
    let inputs = inputs(N, Some(42));
    let result = inputs
        .into_par()
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
