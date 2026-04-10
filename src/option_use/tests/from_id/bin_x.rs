use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::option_use::tests::utils::{UseValue, inputs_opt};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn bin_x_find_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .using_clone(UseValue::new(42))
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
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn bin_x_find_any_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .fallible_option()
        .using_clone(UseValue::new(42))
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
fn bin_x_reduce_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .fallible_option()
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
    assert_eq!(result, Some(Some(160)));
}

#[test]
fn bin_x_reduce_err() {
    let inputs = inputs_opt(N, Some(42));
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .filter_map(|u, x| {
            u.mutate();
            Some(x)
        })
        .fallible_option()
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
    assert_eq!(result, None);
}

#[test_matrix(
    [Vec::new(), SplitVec::with_doubling_growth(), SplitVec::with_linear_growth(6), FixedVec::new(40)],
    [ColIntoMode::Col, ColIntoMode::ColIntoEmpty, ColIntoMode::ColIntoFilled(N / 5)],
    [IterationOrder::Ordered, IterationOrder::Arbitrary]
)]
fn bin_x_collect_ok<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i as u64,
        inputs_opt(N, None)
            .into_iter()
            .map(|x| x.unwrap())
            .filter(|x| x.len() < 4)
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| a + i)
            })
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i as u64) {
        Some(c) => inputs_opt(N, None)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map(|u, x| {
                u.mutate();
                Some(x)
            })
            .fallible_option()
            .filter(|u, x| {
                u.mutate();
                x.len() < 4
            })
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| a + i)
            })
            .iteration_order(order)
            .collect_into(c),
        None => inputs_opt(N, None)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map(|u, x| {
                u.mutate();
                Some(x)
            })
            .fallible_option()
            .filter(|u, x| {
                u.mutate();
                x.len() < 4
            })
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| a + i)
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
fn bin_x_collect_err<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i as u64) {
        Some(c) => inputs_opt(N, Some(42))
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map(|u, x| {
                u.mutate();
                Some(x)
            })
            .fallible_option()
            .filter(|u, x| {
                u.mutate();
                x.len() < 4
            })
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| a + i)
            })
            .iteration_order(order)
            .collect_into(c),
        None => inputs_opt(N, Some(42))
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map(|u, x| {
                u.mutate();
                Some(x)
            })
            .fallible_option()
            .filter(|u, x| {
                u.mutate();
                x.len() < 4
            })
            .flat_map(|u, x| {
                u.mutate();
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| a + i)
            })
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, None);
}
