use crate::option_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::ToString;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn many_m_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .first();
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn many_m_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn many_m_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            match x.as_str() == "7" {
                true => None,
                false => Some(Some(x)),
            }
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, Some(Some(160)));
}

#[test]
fn many_m_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            match x.as_str() == "7" {
                true => None,
                false => Some(match x.as_str() == "42" {
                    true => Some(x),
                    false => None,
                }),
            }
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, None);
}

#[test]
fn many_m_collect_ok() {
    let result: Option<Vec<_>> = inputs(N)
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            match x.as_str() == "7" {
                true => None,
                false => Some(Some(x)),
            }
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .collect();
    assert!(result.is_some());
}

#[test]
fn many_m_collect_err() {
    let result: Option<Vec<_>> = inputs(N)
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            match x.as_str() == "7" {
                true => None,
                false => Some(match x.as_str() == "42" {
                    true => Some(x),
                    false => None,
                }),
            }
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .collect();
    assert!(result.is_none());
}
