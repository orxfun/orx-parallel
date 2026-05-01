use crate::collectables::par_col_into_test::{ColIntoMode, ParCollectIntoTest};
use crate::parameters::IterationOrder;
use crate::result_use::tests::utils::{UseValue, inputs};
use crate::*;
use std::format;
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn many_x_find_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map::<Result<_, Vec<char>>, _>(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Ok(x)),
        })
        .into_fallible()
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
    assert_eq!(result, Ok(Some(String::from("0!"))));
}

#[test]
fn many_x_find_any_ok() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .filter_map::<Result<_, Vec<char>>, _>(|x| match x.as_str() == "7" {
            true => None,
            false => Some(Ok(x)),
        })
        .into_fallible()
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
    assert!(result.is_ok());
}

#[test]
fn many_x_reduce_ok() {
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
        .into_fallible()
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
    assert_eq!(result, Ok(Some(String::from("99!"))));
}

#[test]
fn many_x_reduce_err() {
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
        .into_fallible()
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
    assert_eq!(result, Err(vec!['a']));
}

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn many_x_collect_ok<C: ParCollectIntoTest<String>>(
    _: C,
    mode: ColIntoMode,
    order: IterationOrder,
) {
    let expected = C::expected(
        mode,
        |i| i.to_string(),
        inputs(N)
            .into_iter()
            .filter_map::<Result<_, Vec<char>>, _>(|x| match x.as_str() == "7" {
                true => None,
                false => Some(Ok(x)),
            })
            .map(|x| x.unwrap())
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .flat_map(|x| [format!("{x}!"), x])
            .collect::<std::vec::Vec<_>>(),
    );

    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => inputs(N)
            .into_par()
            .using(|th_idx| UseValue::new(th_idx))
            .filter_map::<Result<_, Vec<char>>, _>(|u, x| {
                u.mutate();
                match x.as_str() == "7" {
                    true => None,
                    false => Some(Ok(x)),
                }
            })
            .into_fallible()
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
            .collect_into(&mut c)
            .map(|_| c),
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
            .into_fallible()
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

#[test_matrix([Vec::new()], [ColIntoMode::Col], [IterationOrder::Ordered])]
fn many_x_collect_err<C: ParCollectIntoTest<String>>(
    _: C,
    mode: ColIntoMode,
    order: IterationOrder,
) {
    let result = match C::init_result(mode, |i| i.to_string()) {
        Some(mut c) => inputs(N)
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
            .into_fallible()
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
            .collect_into(&mut c)
            .map(|_| c),
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
            .into_fallible()
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

    assert_eq!(result, Err(vec!['a']));
}
