use crate::parameters::IterationOrder;
use crate::result_use::tests::utils::{UseValue, inputs};
use crate::*;
use std::string::String;
use std::vec;
use std::vec::Vec;

const N: usize = 157;

#[test]
fn bin_f_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(Ok)
        .into_fallible()
        .use_new(|_| UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .first();
    assert_eq!(result, Ok(Some(String::from("10"))));
}

#[test]
fn bin_f_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(Ok)
        .into_fallible()
        .use_new(|_| UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn bin_f_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Ok(x)
        })
        .into_fallible()
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
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
    assert_eq!(result, Ok(Some(String::from("99"))));
}

#[test]
fn bin_f_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => Ok(x),
                false => Err(vec!['a']),
            }
        })
        .into_fallible()
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
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
    assert_eq!(result, Err(vec!['a']));
}

#[test]
fn bin_f_collect_ok() {
    let result: Result<Vec<_>, Vec<char>> = inputs(N)
        .into_par()
        .use_new(UseValue::new)
        .map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Ok(x)
        })
        .into_fallible()
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .collect();
    assert!(result.is_ok());
}

#[test]
fn bin_f_collect_err() {
    let result: Result<Vec<_>, Vec<char>> = inputs(N)
        .into_par()
        .use_new(UseValue::new)
        .map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => Ok(x),
                false => Err(vec!['a']),
            }
        })
        .into_fallible()
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .collect();
    assert!(result.is_err());
}
