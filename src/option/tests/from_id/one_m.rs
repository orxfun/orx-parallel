use crate::option::tests::utils::inputs_opt;
use crate::parameters::IterationOrder;
use crate::*;

const N: usize = 157;

#[test]
fn one_m_find_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .map(|x| x.parse::<u64>().unwrap())
        .first();
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn one_m_find_any_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .map(|x| x.parse::<u64>().unwrap())
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_m_reduce_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(156)));
}

#[test]
fn one_m_reduce_err() {
    let inputs = inputs_opt(N, Some(42));
    let result = inputs
        .into_par()
        .fallible_option()
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}

#[test]
fn one_m_collect_ok() {
    let inputs = inputs_opt(N, None);
    let result: Option<std::vec::Vec<u64>> = inputs
        .into_par()
        .fallible_option()
        .map(|x| x.parse::<u64>().unwrap())
        .collect::<std::vec::Vec<_>>();
    assert!(result.is_some());
}

#[test]
fn one_m_collect_err() {
    let inputs = inputs_opt(N, Some(42));
    let result: Option<std::vec::Vec<u64>> = inputs
        .into_par()
        .fallible_option()
        .map(|x| x.parse::<u64>().unwrap())
        .collect::<std::vec::Vec<_>>();
    assert_eq!(result, None);
}
