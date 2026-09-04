use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use core::fmt::Debug;
use std::collections::*;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn bin_f_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .into_optional()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .first();
    assert_eq!(result, Some(Some(String::from("10"))));
}

#[test]
fn bin_f_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .into_optional()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn bin_f_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .into_optional()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(String::from("99"))));
}

#[test]
fn bin_f_reduce_err() {
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
        .into_optional()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}

#[test]
fn bin_f_fold_ok() {
    let inputs = inputs(N);
    let expected = inputs
        .clone()
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .into_optional()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .collect::<std::vec::Vec<_>>()
        .unwrap();

    let mut expected_flat = String::new();
    expected.iter().for_each(|x| expected_flat.push_str(x));
    let mut expected: Vec<_> = expected_flat.chars().collect();
    expected.sort();

    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .into_optional()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .num_threads(4)
        .fold(String::new, |s, x| s.push_str(&x));
    let result = result.unwrap();
    assert!(result.len() <= 4);
    let result = result
        .into_iter()
        .reduce(|mut a: String, b: String| {
            a.push_str(&b);
            a
        })
        .unwrap();
    let mut result: Vec<_> = result.chars().collect();
    result.sort();

    assert_eq!(&result, &expected);
}

#[test]
fn bin_f_fold_err() {
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
        .into_optional()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .num_threads(4)
        .fold(String::new, |s, x| s.push_str(&x));
    assert_eq!(result, None);
}

#[test_matrix(
    [Vec::new(), BTreeSet::new(), VecDeque::new()],
    [false, true],
    [IterationOrder::Ordered, IterationOrder::Arbitrary]
)]
fn bin_f_collect_ok<C>(_: C, has_some: bool, order: IterationOrder)
where
    C: ParExtend<String> + Default + Debug + PartialEq + IntoIterator<Item = String>,
{
    let mut expected = C::default();
    if has_some {
        expected.add_one("42".to_string());
        expected.add_one("7".to_string());
    }
    expected.extend(
        inputs(N)
            .into_iter()
            .filter_map(|x| match x.as_str() == "7" {
                true => None,
                false => Some(Some(x)),
            })
            .map(|x| x.unwrap())
            .filter(|x| x.len() > 1)
            .filter(|x| x.len() < 4),
    );

    let par = inputs(N)
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .into_optional()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .iteration_order(order);
    let result = match has_some {
        true => {
            let mut result = C::default();
            result.add_one("42".to_string());
            result.add_one("7".to_string());
            par.collect_into(&mut result).map(|_| result)
        }
        false => par.collect(),
    };

    let result = result.unwrap();
    match order {
        IterationOrder::Ordered => assert_eq!(expected, result),
        IterationOrder::Arbitrary => {
            let mut expected: Vec<_> = expected.into_iter().collect();
            let mut result: Vec<_> = result.into_iter().collect();
            expected.sort();
            result.sort();
            assert_eq!(expected, result);
        }
    }
}

#[test_matrix(
    [Vec::new(), BTreeSet::new(), VecDeque::new()],
    [false, true],
    [IterationOrder::Ordered, IterationOrder::Arbitrary]
)]
fn bin_f_collect_err<C>(_: C, has_some: bool, order: IterationOrder)
where
    C: ParExtend<String> + Default,
{
    let par = inputs(N)
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(match x.as_str() == "42" {
                true => Some(x),
                false => None,
            }),
        })
        .into_optional()
        .filter(|x| x.len() > 1)
        .filter(|x| x.len() < 4)
        .iteration_order(order);
    let result = match has_some {
        true => {
            let mut result = C::default();
            result.add_one("42".to_string());
            result.add_one("7".to_string());
            par.collect_into(&mut result)
        }
        false => {
            let result: Option<C> = par.collect();
            result.map(|_| ())
        }
    };

    assert_eq!(result, None);
}
