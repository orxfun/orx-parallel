use crate::infallible_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use std::string::String;

#[cfg(not(miri))]const N: usize = 157;#[cfg(miri)]const N: usize = 57;

#[test]
fn id_find() {
    let inputs = inputs(N);
    let result = inputs.into_par().using_clone(UseValue::new(42)).first();
    assert_eq!(result, Some(String::from("0")));
}

#[test]
fn id_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn id_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using_clone(UseValue::new(42))
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, Some(String::from("99")));
}
