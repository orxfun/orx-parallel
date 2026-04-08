use crate::infallible::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;

const N: usize = 157;

#[test]
fn bin_m_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .first();
    assert_eq!(result, Some(0));
}

#[test]
fn bin_m_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn bin_m_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(156));
}
