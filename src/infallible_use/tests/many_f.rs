use crate::infallible_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use std::string::{String, ToString};

#[cfg(not(miri))]const N: usize = 157;#[cfg(miri)]const N: usize = 57;

#[test]
fn many_f_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using_clone(UseValue::new(42))
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
    assert_eq!(result, Some(String::from("0")));
}

#[test]
fn many_f_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
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
fn many_f_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using_clone(UseValue::new(42))
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
    assert_eq!(result, Some(String::from("99")));
}
