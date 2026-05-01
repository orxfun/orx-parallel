use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::parameters::IterationOrder;
use crate::result_use::tests::utils::{UseValue, inputs_res};
use crate::*;
use std::vec;
use std::vec::Vec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn bin_x_find_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .into_fallible()
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
    assert_eq!(result, Ok(Some(0)));
}

#[test]
fn bin_x_find_any_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .into_fallible()
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
    assert!(result.is_ok());
}

#[test]
fn bin_x_reduce_ok() {
    let inputs = inputs_res(N, None);
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_fallible()
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
    assert_eq!(result, Ok(Some(160)));
}

#[test]
fn bin_x_reduce_err() {
    let inputs = inputs_res(N, Some(42));
    let result = inputs
        .into_par()
        .using(|th_idx| UseValue::new(th_idx))
        .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
            u.mutate();
            Some(x)
        })
        .into_fallible()
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
    assert_eq!(result, Err(vec!['a']));
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn bin_x_collect_ok<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let expected = C::expected(
        mode,
        |i| i as u64,
        inputs_res(N, None)
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
        Some(mut c) => inputs_res(N, None)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
                u.mutate();
                Some(x)
            })
            .into_fallible()
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
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs_res(N, None)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
                u.mutate();
                Some(x)
            })
            .into_fallible()
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

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn bin_x_collect_err<C: ParCollectIntoTest<u64>>(_: C, mode: ColIntoMode, order: IterationOrder) {
    let result = match C::init_result(mode, |i| i as u64) {
        Some(mut c) => inputs_res(N, Some(42))
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
                u.mutate();
                Some(x)
            })
            .into_fallible()
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
            .collect_into(&mut c)
            .map(|_| c),
        None => inputs_res(N, Some(42))
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
                u.mutate();
                Some(x)
            })
            .into_fallible()
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

    assert_eq!(result, Err(vec!['a']));
}
