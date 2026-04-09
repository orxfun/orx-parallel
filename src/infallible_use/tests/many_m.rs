use crate::infallible_use::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use std::string::ToString;

const N: usize = 157;

#[test]
fn many_m_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|x| x.parse::<u64>().unwrap())
        .first();
    assert_eq!(result, Some(0));
}

#[test]
fn many_m_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|x| x.parse::<u64>().unwrap())
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn many_m_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(160));
}
