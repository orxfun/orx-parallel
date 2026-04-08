use crate::parameters::IterationOrder;
use crate::result_depr2::tests::utils::inputs;
use crate::*;
use std::string::ToString;
use std::vec;

const N: usize = 157;

#[test]
fn many_m_find_ok() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|x| x.parse::<u64>().unwrap())
        .first();
    assert_eq!(result, Ok(Some(0)));
}

#[test]
fn many_m_find_any_ok() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|x| x.parse::<u64>().unwrap())
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn many_m_reduce_ok() {
    let inputs = inputs(N, None);
    let result = inputs
        .into_par()
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Ok(Some(160)));
}

#[test]
fn many_m_reduce_err() {
    let inputs = inputs(N, Some(42));
    let result = inputs
        .into_par()
        .fallible_result()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Err(vec!['a']));
}
