use crate::option_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;

const N: usize = 157;

#[test]
fn one_m_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(Some)
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .first();
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn one_m_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(Some)
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_m_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
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
    assert_eq!(result, Some(Some(156)));
}

#[test]
fn one_m_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => Some(x),
                false => None,
            }
        })
        .into_optional()
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
fn one_m_collect_ok() {
    let result: Option<Vec<_>> = inputs(N)
        .into_par()
        .use_new(UseValue::new)
        .map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .collect();
    assert!(result.is_some());
}

#[test]
fn one_m_collect_err() {
    let result: Option<Vec<_>> = inputs(N)
        .into_par()
        .use_new(UseValue::new)
        .map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => Some(x),
                false => None,
            }
        })
        .into_optional()
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .collect();
    assert!(result.is_none());
}
