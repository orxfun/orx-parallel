use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::infallible_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn bin_x_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(|_| UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .first();
    assert_eq!(result, Some(0));
}

#[test]
fn bin_x_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(|th_idx| UseValue::new(th_idx))
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
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
fn bin_x_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(|_| UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
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
    assert_eq!(result, Some(160));
}

#[test]
fn bin_x_fold() {
    let inputs = inputs(N);

    let mut expected: Vec<_> = inputs
        .iter()
        .filter(|x| x.len() < 4)
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .collect();
    expected.sort_unstable();

    let par = inputs
        .into_par()
        .num_threads(4)
        .use_new(|_| UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() < 4
        })
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        });
    let result = par.fold(Vec::new, |u, v, x| {
        u.mutate();
        v.push(x);
    });
    assert!(result.len() <= 4);
    let result = result
        .into_iter()
        .reduce(|mut a: Vec<u64>, mut b: Vec<u64>| {
            a.append(&mut b);
            a
        })
        .unwrap();
    let mut result = result;
    result.sort_unstable();

    assert_eq!(&result, &expected);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn bin_x_collect<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let iter = || {
        inputs(N).into_iter().filter(|x| x.len() < 4).flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
    };

    let expected = C::expected(mode, |i| i.to_string(), iter());

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => {
            inputs(N)
                .into_par()
                .use_new(|th_idx| UseValue::new(th_idx))
                .iteration_order(order)
                .filter(|u, x| {
                    u.mutate();
                    x.len() < 4
                })
                .flat_map(|u, x| {
                    u.mutate();
                    let a = x.parse::<u64>().unwrap();
                    (0..5).map(move |i| (a + i).to_string())
                })
                .collect_into(&mut c);
            c
        }
        None => inputs(N)
            .into_par()
            .use_new(|th_idx| UseValue::new(th_idx))
            .iteration_order(order)
            .filter(|u, x| {
                u.mutate();
                x.len() < 4
            })
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .collect(),
    };

    C::assert_eq(result, expected, order);
}
