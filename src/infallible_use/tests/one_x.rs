use crate::infallible_use::tests::utils::{UseValue, inputs};
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use core::fmt::Debug;
use std::collections::*;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn one_x_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(|_| UseValue::new(42))
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| a + i)
        })
        .first();
    assert_eq!(result, Some(0));
}

#[test]
fn one_x_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
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
fn one_x_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(|_| UseValue::new(42))
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
fn one_x_fold() {
    let inputs = inputs(N);

    let mut expected: Vec<_> = inputs
        .iter()
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

#[test_matrix(
    [Vec::new(), BTreeSet::new(), VecDeque::new()],
    [false, true],
    [IterationOrder::Ordered, IterationOrder::Arbitrary]
)]
fn one_x_collect<C>(_: C, has_some: bool, order: IterationOrder)
where
    C: ParExtend<String> + Default + Debug + PartialEq + IntoIterator<Item = String>,
{
    let iter = || {
        inputs(N).into_iter().flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
    };

    let mut expected = C::default();
    if has_some {
        expected.add_one("42".to_string());
        expected.add_one("7".to_string());
    }
    expected.extend(iter());

    let par = inputs(N)
        .into_par()
        .use_new(UseValue::new)
        .iteration_order(order)
        .flat_map(|u, x| {
            u.mutate();
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        });
    let result = match has_some {
        true => {
            let mut result = C::default();
            result.add_one("42".to_string());
            result.add_one("7".to_string());
            par.collect_into(&mut result);
            result
        }
        false => par.collect(),
    };

    match order {
        IterationOrder::Ordered => assert_eq!(expected, result),
        IterationOrder::Arbitrary => {
            let mut expected: Vec<_> = expected.into_iter().collect();
            let mut result: Vec<_> = result.into_iter().collect();
            expected.sort();
            result.sort();
            assert_eq!(expected, result);
        }
    }
}
