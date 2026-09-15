use crate::parameters::IterationOrder;
use crate::result_use::tests::utils::{UseValue, inputs_res};
use crate::*;
use std::string::String;
use std::vec;
use std::vec::Vec;

const N: usize = 157;

#[test]
fn id_find_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .into_fallible()
        .use_new(|_| UseValue::new(42))
        .first();
    assert_eq!(result, Ok(Some(String::from("0"))));
}

#[test]
fn id_find_any_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .into_fallible()
        .use_new(|_| UseValue::new(42))
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn id_reduce_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_fallible()
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, Ok(Some(String::from("99"))));
}

#[test]
fn id_reduce_ok_err() {
    let inputs = inputs_res(N, Some(42));
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_fallible()
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, Err(vec!['a']));
}

#[test]
fn id_collect_ok() {
    let result: Result<Vec<_>, Vec<char>> = inputs_res(N, None)
        .into_par()
        .use_new(UseValue::new)
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_fallible()
        .collect();
    assert!(result.is_ok());
}

#[test]
fn id_collect_err() {
    let result: Result<Vec<_>, Vec<char>> = inputs_res(N, Some(42))
        .into_par()
        .use_new(UseValue::new)
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_fallible()
        .collect();
    assert!(result.is_err());
}
