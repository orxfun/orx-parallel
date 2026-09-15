use crate::parameters::IterationOrder;
use crate::result_use::tests::utils::{UseValue, inputs_res};
use crate::*;
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

const N: usize = 157;

#[test]
fn many_f_find_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .into_fallible()
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
    assert_eq!(result, Ok(Some(String::from("0"))));
}

#[test]
fn many_f_find_any_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .into_fallible()
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
    assert!(result.is_ok());
}

#[test]
fn many_f_reduce_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_fallible()
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
    assert_eq!(result, Ok(Some(String::from("99"))));
}

#[test]
fn many_f_reduce_err() {
    let inputs = inputs_res(N, Some(42));
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_fallible()
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
    assert_eq!(result, Err(vec!['a']));
}

#[test]
fn many_f_collect_ok() {
    let result: Result<Vec<_>, Vec<char>> = inputs_res(N, None)
        .into_par()
        .use_new(UseValue::new)
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_fallible()
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
    assert!(result.is_ok());
}

#[test]
fn many_f_collect_err() {
    let result: Result<Vec<_>, Vec<char>> = inputs_res(N, Some(42))
        .into_par()
        .use_new(UseValue::new)
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_fallible()
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
    assert!(result.is_err());
}
