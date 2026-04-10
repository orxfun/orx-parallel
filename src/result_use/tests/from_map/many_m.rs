use crate::parameters::IterationOrder;
use crate::result_use::tests::utils::{UseValue, inputs};
use crate::*;
use std::string::ToString;
use std::vec;
use std::vec::Vec;

#[cfg(not(miri))]
const N: usize = 257;
#[cfg(miri)]
const N: usize = 57;

#[test]
fn many_m_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
        .fallible_result()
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
    assert_eq!(result, Ok(Some(0)));
}

#[test]
fn many_m_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .map::<Result<_, Vec<char>>, _>(|x| Ok(x))
        .fallible_result()
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
    assert!(result.is_ok());
}

#[test]
fn many_m_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Ok(x)
        })
        .fallible_result()
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
    assert_eq!(result, Ok(Some(160)));
}

#[test]
fn many_m_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => Ok(x),
                false => Err(vec!['a']),
            }
        })
        .fallible_result()
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
    assert_eq!(result, Err(vec!['a']));
}
