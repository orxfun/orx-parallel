use crate::option_use::tests::utils::{UseValue, inputs_opt};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::{String, ToString};

const N: usize = 157;

#[test]
fn many_f_find_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .first();
    assert_eq!(result, Some(Some(String::from("0"))));
}

#[test]
fn many_f_find_any_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn many_f_reduce_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, Some(Some(String::from("99"))));
}

#[test]
fn many_f_reduce_err() {
    let inputs = inputs_opt(N, Some(42));
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
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
fn many_f_collect_ok() {
    let result: Option<Vec<_>> = inputs_opt(N, None)
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .collect();
    assert!(result.is_some());
}

#[test]
fn many_f_collect_err() {
    let result: Option<Vec<_>> = inputs_opt(N, Some(42))
        .into_par()
        .use_new(UseValue::new)
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .collect();
    assert!(result.is_none());
}
