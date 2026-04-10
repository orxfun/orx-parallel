use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::parameters::IterationOrder;
use crate::result_use::tests::utils::{UseValue, inputs};
use crate::*;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use std::vec;
use std::vec::Vec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn one_m_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map::<Result<_, Vec<char>>, _>(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Ok(x)),
        })
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
        .filter_map::<Result<_, Vec<char>>, _>(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Ok(x)),
        })
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
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            match x.as_str() == "7" {
                true => None,
                false => Some(Ok(x)),
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
    assert_eq!(result, Ok(Some(156)));
}

#[test]
fn one_m_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            match x.as_str() == "7" {
                true => None,
                false => Some(match x.as_str() == "42" {
                    true => Ok(x),
                    false => Err(vec!['a']),
                }),
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

#[test_matrix(
    [Vec::new(), SplitVec::with_doubling_growth(), SplitVec::with_linear_growth(6), FixedVec::new(40)],
    [ColIntoMode::Col, ColIntoMode::ColIntoEmpty, ColIntoMode::ColIntoFilled(N / 5)],
    [IterationOrder::Ordered, IterationOrder::Arbitrary]
)]
fn one_m_collect_ok<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i as u64,
        inputs(N)
            .into_iter()
            .filter_map::<Result<_, Vec<char>>, _>(|x| match x.as_str() == "7" {
                true => None,
                false => Some(Ok(x)),
            })
            .map(|x| x.unwrap())
            .map(|x| x.parse::<u64>().unwrap())
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i as u64) {
        Some(c) => inputs(N)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
                u.mutate();
                match x.as_str() == "7" {
                    true => None,
                    false => Some(Ok(x)),
                }
            })
            .fallible_result()
            .map(|u, x| {
                u.mutate();
                x.parse::<u64>().unwrap()
            })
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
                u.mutate();
                match x.as_str() == "7" {
                    true => None,
                    false => Some(Ok(x)),
                }
            })
            .fallible_result()
            .map(|u, x| {
                u.mutate();
                x.parse::<u64>().unwrap()
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
fn one_m_collect_err<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i as u64) {
        Some(c) => inputs(N)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
                u.mutate();
                match x.as_str() == "7" {
                    true => None,
                    false => Some(match x.as_str() == "42" {
                        true => Ok(x),
                        false => Err(vec!['a']),
                    }),
                }
            })
            .fallible_result()
            .map(|u, x| {
                u.mutate();
                x.parse::<u64>().unwrap()
            })
            .iteration_order(order)
            .collect_into(c),
        None => inputs(N)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
                u.mutate();
                match x.as_str() == "7" {
                    true => None,
                    false => Some(match x.as_str() == "42" {
                        true => Ok(x),
                        false => Err(vec!['a']),
                    }),
                }
            })
            .fallible_result()
            .map(|u, x| {
                u.mutate();
                x.parse::<u64>().unwrap()
            })
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, Err(vec!['a']));
}
