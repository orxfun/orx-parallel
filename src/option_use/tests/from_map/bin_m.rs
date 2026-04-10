use crate::option_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;

const N: usize = 157;

#[test]
fn bin_m_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| Some(x))
        .fallible_option()
        .using_clone(UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .first();
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn bin_m_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| Some(x))
        .fallible_option()
        .using_clone(UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
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
fn bin_m_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .map(|u, x| {
            u.mutate();
            Some(x)
        })
        .fallible_option()
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
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
    assert_eq!(result, Some(Some(156)));
}

#[test]
fn bin_m_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => Some(x),
                false => None,
            }
        })
        .fallible_option()
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
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
