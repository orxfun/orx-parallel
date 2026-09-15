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
fn one_f_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(|_| UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .first();
    assert_eq!(result, Some(String::from("10")));
}

#[test]
fn one_f_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(UseValue::new)
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn one_f_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .use_new(|_| UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
        })
        .reduce(|u, a, b| {
            u.mutate();
            match a < b {
                true => b,
                false => a,
            }
        });
    assert_eq!(result, Some(String::from("99")));
}

#[test]
fn one_f_fold() {
    let inputs = inputs(N);

    let mut expected = String::new();
    inputs
        .iter()
        .filter(|x| x.len() > 1)
        .for_each(|x| expected.push_str(x));
    let mut expected: Vec<_> = expected.chars().collect();
    expected.sort();

    let par = inputs
        .into_par()
        .num_threads(4)
        .use_new(|_| UseValue::new(42))
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
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

#[test_matrix(
    [Vec::new(), BTreeSet::new(), VecDeque::new()],
    [false, true],
    [IterationOrder::Ordered, IterationOrder::Arbitrary]
)]
fn one_f_collect<C>(_: C, has_some: bool, order: IterationOrder)
where
    C: ParExtend<String> + Default + Debug + PartialEq + IntoIterator<Item = String>,
{
    let iter = || inputs(N).into_iter().filter(|x| x.len() > 1);

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
        .filter(|u, x| {
            u.mutate();
            x.len() > 1
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
