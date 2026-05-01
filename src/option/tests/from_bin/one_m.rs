use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::option::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn one_m_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .into_optional()
        .map(|x| x.parse::<u64>().unwrap())
        .first();
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn one_m_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .into_optional()
        .map(|x| x.parse::<u64>().unwrap())
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_m_reduce_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Some(x)),
        })
        .into_optional()
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(156)));
}

#[test]
fn one_m_reduce_err() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map(|x| match x.as_str() == "7" {
            true => None,
            false => Some(match x.as_str() == "42" {
                true => Some(x),
                false => None,
            }),
        })
        .into_optional()
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn one_m_collect_ok<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i as u64,
        inputs(N)
            .into_iter()
            .filter_map(|x| match x.as_str() == "7" {
                true => None,
                false => Some(Some(x)),
            })
            .map(|x| x.unwrap())
            .map(|x| x.parse::<u64>().unwrap())
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i as u64) {
        Some(mut c) => inputs(N)
            .into_par()
            .filter_map(|x| match x.as_str() == "7" {
                true => None,
                false => Some(Some(x)),
            })
            .into_optional()
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs(N)
            .into_par()
            .filter_map(|x| match x.as_str() == "7" {
                true => None,
                false => Some(Some(x)),
            })
            .into_optional()
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect(),
    };

    C::assert_eq(result.unwrap(), expected, order);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn one_m_collect_err<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i as u64) {
        Some(mut c) => inputs(N)
            .into_par()
            .filter_map(|x| match x.as_str() == "7" {
                true => None,
                false => Some(match x.as_str() == "42" {
                    true => Some(x),
                    false => None,
                }),
            })
            .into_optional()
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs(N)
            .into_par()
            .filter_map(|x| match x.as_str() == "7" {
                true => None,
                false => Some(match x.as_str() == "42" {
                    true => Some(x),
                    false => None,
                }),
            })
            .into_optional()
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, None);
}
