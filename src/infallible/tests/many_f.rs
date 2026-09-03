use crate::infallible::tests::utils::inputs;
use crate::parameters::IterationOrder;
use crate::*;
use alloc::vec::Vec;
use core::fmt::Debug;
use std::string::{String, ToString};
use test_case::test_matrix;

const N: usize = 157;

#[test]
fn many_f_find() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .first();
    assert_eq!(result, Some(String::from("0")));
}

#[test]
fn many_f_find_any() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .iteration_order(IterationOrder::Arbitrary)
        .first();
    assert!(result.is_some());
}

#[test]
fn many_f_reduce() {
    let inputs = inputs(N);
    let result = inputs
        .into_par()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .reduce(|a, b| match a < b {
            true => b,
            false => a,
        });
    assert_eq!(result, Some(String::from("99")));
}

#[test]
fn many_f_fold() {
    let inputs = inputs(N);

    let mut expected = String::new();
    inputs
        .iter()
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4)
        .for_each(|x| expected.push_str(&x));
    let mut expected: Vec<_> = expected.chars().collect();
    expected.sort();

    let par = inputs
        .into_par()
        .num_threads(4)
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4);
    let result = par.fold(String::new, |s, x| s.push_str(&x));
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

#[test_matrix([Vec::new()], [false, true], [IterationOrder::Ordered])]
fn many_f_collect<C>(_: C, has_some: bool, order: IterationOrder)
where
    C: ParExtend<String> + Default + Debug + PartialEq + IntoIterator<Item = String>,
{
    let iter = || {
        inputs(N)
            .into_iter()
            .flat_map(|x| {
                let a = x.parse::<u64>().unwrap();
                (0..5).map(move |i| (a + i).to_string())
            })
            .filter(|x| x.len() < 4)
    };

    let mut expected = C::default();
    if has_some {
        expected.add_one("42".to_string());
        expected.add_one("7".to_string());
    }
    expected.extend(iter().map(|i| i.to_string()));

    let par = inputs(N)
        .into_par()
        .iteration_order(order)
        .flat_map(|x| {
            let a = x.parse::<u64>().unwrap();
            (0..5).map(move |i| (a + i).to_string())
        })
        .filter(|x| x.len() < 4);
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
