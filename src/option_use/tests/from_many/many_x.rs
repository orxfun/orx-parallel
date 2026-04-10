use crate::option_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use std::format;
use std::string::{String, ToString};
use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn many_x_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
        .fallible_option()
        .using_clone(UseValue::new(42))
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
    assert_eq!(result, Some(Some(String::from("0!"))));
}

#[test]
fn many_x_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
        .fallible_option()
        .using_clone(UseValue::new(42))
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
fn many_x_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .flat_map(|u, x| {
            u.mutate();
            [x.clone(), x.clone(), x].map(Some)
        })
        .fallible_option()
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
    assert_eq!(result, Some(Some(String::from("99!"))));
}

#[test]
fn many_x_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .flat_map(|u, x| {
            u.mutate();
            match x.as_str() == "42" {
                true => [x.clone(), x.clone(), x].map(Some),
                false => [None, None, None],
            }
        })
        .fallible_option()
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
    assert_eq!(result, None);
}


#[test_matrix(
    [Vec::new(), SplitVec::with_doubling_growth(), SplitVec::with_linear_growth(6), FixedVec::new(40)],
    [ColIntoMode::Col, ColIntoMode::ColIntoEmpty, ColIntoMode::ColIntoFilled(N / 5)],
    [IterationOrder::Ordered, IterationOrder::Arbitrary]
)]
fn many_x_collect_ok<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i.to_string(),
            inputs(N)
                .into_iter()
                .flat_map(|x| [x.clone(), x.clone(), x].map(Some))
            .map(|x| x.unwrap())
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .flat_map(|x| [format!("{x}!"), x])
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(c) =>             inputs(N)
                .into_par()
                .using(|th_idx| UseValue::new(th_idx))
                .flat_map(|u, x| {
                    u.mutate();
                    [x.clone(), x.clone(), x].map(Some)
                })
            .fallible_option()
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .flat_map(|u, x| {
                u.mutate();
                [format!("{x}!"), x]
            })
            .iteration_order(order)
            .collect_into(c),
        None =>             inputs(N)
                .into_par()
                .using(|th_idx| UseValue::new(th_idx))
                .flat_map(|u, x| {
                    u.mutate();
                    [x.clone(), x.clone(), x].map(Some)
                })
            .fallible_option()
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .flat_map(|u, x| {
                u.mutate();
                [format!("{x}!"), x]
            })
            .iteration_order(order)
            .collect(),
    };

    C::assert_eq(result.unwrap(), expected, order);
}


#[test_matrix(
    [Vec::new(), SplitVec::with_doubling_growth(), SplitVec::with_linear_growth(6), FixedVec::new(40)],
    [ColIntoMode::Col, ColIntoMode::ColIntoEmpty, ColIntoMode::ColIntoFilled(N / 5)],
    [IterationOrder::Ordered, IterationOrder::Arbitrary]
)]
fn many_x_collect_err<C: ParCollectIntoTest<String>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(c) =>             inputs(N)
                .into_par()
                .using(|th_idx| UseValue::new(th_idx))
                .flat_map(|u, x| {
                    u.mutate();
                    match x.as_str() == "42" {
                        true => [x.clone(), x.clone(), x].map(Some),
                        false => [None, None, None],
                    }
                })
            .fallible_option()
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .flat_map(|u, x| {
                u.mutate();
                [format!("{x}!"), x]
            })
            .iteration_order(order)
            .collect_into(c),
        None =>             inputs(N)
                .into_par()
                .using(|th_idx| UseValue::new(th_idx))
                .flat_map(|u, x| {
                    u.mutate();
                    match x.as_str() == "42" {
                        true => [x.clone(), x.clone(), x].map(Some),
                        false => [None, None, None],
                    }
                })
            .fallible_option()
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .flat_map(|u, x| {
                u.mutate();
                [format!("{x}!"), x]
            })
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, None);
}
