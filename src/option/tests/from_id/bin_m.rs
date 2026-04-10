use crate::option::tests::utils::inputs_opt;
use crate::parameters::IterationOrder;
use crate::*;

#[cfg(not(miri))]const N: usize = 157;#[cfg(miri)]const N: usize = 57;

#[test]
fn bin_m_find_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .first();
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn bin_m_find_any_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn bin_m_reduce_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(156)));
}

#[test]
fn bin_m_reduce_err() {
    let inputs = inputs_opt(N, Some(42));
    let result = inputs
        .into_par()
        .fallible_option()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}
