use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::infallible_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use std::format;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn many_x_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(|_| UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|u, x| {
            u.mutate();
            [format!("{x}!"), x]
        })
        .first();
    assert_eq!(result, Some(String::from("0!")));
}

#[test]
fn many_x_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(|th_idx| UseValue::new(th_idx))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|u, x| {
            u.mutate();
            [format!("{x}!"), x]
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn many_x_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(|_| UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|u, x| {
            u.mutate();
            [format!("{x}!"), x]
        })
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, Some(String::from("99!")));
}

#[test]
fn many_x_fold() {
    let inputs = inputs(N);

    let mut expected = String::new();
    inputs
        .iter()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|x| [format!("{x}!"), x])
        .for_each(|x| expected.push_str(&x));
    let mut expected: Vec<_> = expected.chars().collect();
    expected.sort();

    let par = inputs
        .into_par()
        .num_threads(4)
        .use_new(|_| UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .flat_map(|u, x| {
            u.mutate();
            [format!("{x}!"), x]
        });
    let result = par.fold(String::new, |u, s, x| {
        u.mutate();
        s.push_str(&x);
    });
    assert!(result.len() <= 4);
    let result = result
        .into_iter()
        .reduce(|mut a: String, b: String| {
            a.push_str(&b);
            a
        })
        .unwrap();
    let mut result: Vec<_> = result.chars().collect();
    result.sort();

    assert_eq!(&result, &expected);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn many_x_collect<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let iter = || {
        inputs(N)
            .into_iter()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .flat_map(|x| [format!("{x}!"), x])
    };

    let expected = C::expected(mode, |i| i.to_string(), iter());

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => {
            inputs(N)
                .into_par()
                .use_new(|th_idx| UseValue::new(th_idx))
                .iteration_order(order)
                .flat_map(|u, x| {
                    u.mutate();
                    let a = x.parse::<u64>().unwrap();
                    (0..5).map(move |i| (a + i).to_string())
                })
                .flat_map(|u, x| {
                    u.mutate();
                    [format!("{x}!"), x]
                })
                .collect_into(&mut c);
            c
        }
        None => inputs(N)
            .into_par()
            .use_new(|th_idx| UseValue::new(th_idx))
            .iteration_order(order)
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .flat_map(|u, x| {
                u.mutate();
                [format!("{x}!"), x]
            })
            .collect(),
    };

    C::assert_eq(result, expected, order);
}
