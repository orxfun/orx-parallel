use crate::option_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use std::format;
use std::string::{String, ToString};

const N: usize = 157;

#[test]
fn many_x_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| Some(x))
        .fallible_option()
        .using_clone(UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|u, x| {
            u.mutate();
            [format!("{x}!"), x]
        })
        .first();
    assert_eq!(result, Some(Some(String::from("0!"))));
}

#[test]
fn many_x_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map(|x| Some(x))
        .fallible_option()
        .using_clone(UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|u, x| {
            u.mutate();
            [format!("{x}!"), x]
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn many_x_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .map(|u, x| {
            u.mutate();
            Some(x)
        })
        .fallible_option()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|u, x| {
            u.mutate();
            [format!("{x}!"), x]
        })
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, Some(Some(String::from("99!"))));
}

#[test]
fn many_x_reduce_err() {
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
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|u, x| {
            u.mutate();
            [format!("{x}!"), x]
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
