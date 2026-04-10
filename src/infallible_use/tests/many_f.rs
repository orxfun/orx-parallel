use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::infallible_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

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

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn many_f_collect<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let iter = || {
        inputs(N)
            .into_iter()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|x| x.len() < 4)
    };

    let expected = C::expected(mode, |i| i.to_string(), iter());

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(c) => inputs(N)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .iteration_order(order)
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|u, x| {
                u.mutate();
                x.len() < 4
            })
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .iteration_order(order)
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|u, x| {
                u.mutate();
                x.len() < 4
            })
            .collect(),
    };

    C::assert_eq(result, expected, order);
}
