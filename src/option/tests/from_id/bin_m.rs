use crate::option::tests::utils::inputs_opt;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;

const N: usize = 157;

#[test]
fn bin_m_find_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
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
        .into_optional()
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
        .into_optional()
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
        .into_optional()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}

#[test]
fn bin_m_fold_ok() {
    let inputs = inputs_opt(N, None);
    let mut expected = inputs
        .clone()
        .into_par()
        .into_optional()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .collect::<std::vec::Vec<_>>()
        .unwrap();
    expected.sort_unstable();

    let result = inputs
        .into_par()
        .into_optional()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
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
fn bin_m_fold_err() {
    let inputs = inputs_opt(N, Some(42));
    let result = inputs
        .into_par()
        .into_optional()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .num_threads(4)
        .fold(Vec::new, |v, x| v.push(x));
    assert_eq!(result, None);
}

#[test]
fn bin_m_collect_ok() {
    let result: Option<Vec<_>> = inputs_opt(N, None)
        .into_par()
        .into_optional()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .collect();
    assert!(result.is_some());
}

#[test]
fn bin_m_collect_err() {
    let result: Option<Vec<_>> = inputs_opt(N, Some(42))
        .into_par()
        .into_optional()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .collect();
    assert!(result.is_none());
}
