use crate::option_use::tests::utils::{UseValue, inputs_res};
use crate::parameters::IterationOrder;
use crate::*;
use std::string::ToString;

const N: usize = 157;

#[test]
fn many_m_find_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .using_clone(UseValue::new(42))
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
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .using_clone(UseValue::new(42))
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
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .fallible_option()
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
    let inputs = inputs_res(N, Some(42));
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .fallible_option()
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
