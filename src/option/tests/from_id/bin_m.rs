use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::option::tests::utils::inputs_opt;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn bin_m_find_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .first();
    assert_eq!(result, Some(Some(0)));
}

#[test]
fn bin_m_find_any_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn bin_m_reduce_ok() {
    let inputs = inputs_opt(N, None);
    let result = inputs
        .into_par()
        .into_optional()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(Some(156)));
}

#[test]
fn bin_m_reduce_err() {
    let inputs = inputs_opt(N, Some(42));
    let result = inputs
        .into_par()
        .into_optional()
        .filter(|x| x.len() < 4)
        .map(|x| x.parse::<u64>().unwrap())
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, None);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn bin_m_collect_ok<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i as u64,
        inputs_opt(N, None)
            .into_iter()
            .map(|x| x.unwrap())
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i as u64) {
        Some(mut c) => inputs_opt(N, None)
            .into_par()
            .into_optional()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs_opt(N, None)
            .into_par()
            .into_optional()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect(),
    };

    C::assert_eq(result.unwrap(), expected, order);
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn bin_m_collect_err<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i as u64) {
        Some(mut c) => inputs_opt(N, Some(42))
            .into_par()
            .into_optional()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs_opt(N, Some(42))
            .into_par()
            .into_optional()
            .filter(|x| x.len() < 4)
            .map(|x| x.parse::<u64>().unwrap())
            .iteration_order(order)
            .collect(),
    };

    assert_eq!(result, None);
}
