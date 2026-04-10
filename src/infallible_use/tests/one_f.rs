use crate::infallible_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use std::string::String;

#[cfg(not(miri))]const N: usize = 157;#[cfg(miri)]const N: usize = 57;

#[test]
fn one_f_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using_clone(UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .first();
    assert_eq!(result, Some(String::from("10")));
}

#[test]
fn one_f_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_f_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using_clone(UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
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
