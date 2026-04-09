use crate::parameters::IterationOrder;
use crate::result_use::tests::utils::{UseValue, inputs};
use crate::*;
use std::vec;
use std::vec::Vec;

const N: usize = 157;

#[test]
fn one_m_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok))
        .fallible_result()
        .using_clone(UseValue::new(42))
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .first();
    assert_eq!(result, Ok(Some(0)));
}

#[test]
fn one_m_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok))
        .fallible_result()
        .using_clone(UseValue::new(42))
        .map(|u, x| {
            u.mutate();
            x.parse::<u64>().unwrap()
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_ok());
}

#[test]
fn one_m_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .flat_map(|u, x| {
            u.mutate();
            [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok)
        })
        .fallible_result()
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
    assert_eq!(result, Ok(Some(156)));
}

#[test]
fn one_m_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .flat_map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => [x.clone(), x.clone(), x].map(Result::<_, Vec<char>>::Ok),
                false => [Err(vec!['a']), Err(vec!['b']), Err(vec!['c'])],
            }
        })
        .fallible_result()
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
    assert_eq!(result, Err(vec!['a']));
}
