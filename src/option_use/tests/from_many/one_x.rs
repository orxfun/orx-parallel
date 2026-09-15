use crate::option_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;

const N: usize = 157;

#[test]
fn one_x_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .first();
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn one_x_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
        .into_optional()
        .use_new(|_| UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_x_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .flat_map(|u, x| {
            u.mutate();
            [x.clone(), x.clone(), x].map(Some)
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
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
fn one_x_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .flat_map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => [x.clone(), x.clone(), x].map(Some),
                false => [None, None, None],
            }
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
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

#[test]
fn one_x_collect_ok() {
    let result: Option<Vec<_>> = inputs(N)
        .into_par()
        .use_new(UseValue::new)
        .flat_map(|u, x| {
            u.mutate();
            [x.clone(), x.clone(), x].map(Some)
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .collect();
    assert!(result.is_some());
}

#[test]
fn one_x_collect_err() {
    let result: Option<Vec<_>> = inputs(N)
        .into_par()
        .use_new(UseValue::new)
        .flat_map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => [x.clone(), x.clone(), x].map(Some),
                false => [None, None, None],
            }
        })
        .into_optional()
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .collect();
    assert!(result.is_none());
}
