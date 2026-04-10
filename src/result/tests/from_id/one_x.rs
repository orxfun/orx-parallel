use crate::parameters::IterationOrder;
use crate::result::tests::utils::inputs_res;
use crate::*;
use std::vec;

#[cfg(not(miri))]const N: usize = 157;#[cfg(miri)]const N: usize = 57;

#[test]
fn one_x_find_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .first();
    assert_eq!(result, Ok(Some(0)));
}

#[test]
fn one_x_find_any_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn one_x_reduce_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Ok(Some(160)));
}

#[test]
fn one_x_reduce_err() {
    let inputs = inputs_res(N, Some(42));
    let result = inputs
        .into_par()
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Err(vec!['a']));
}
