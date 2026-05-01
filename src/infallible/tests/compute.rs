use crate::infallible::tests::utils::inputs;
use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::string::{String, ToString};

const N: usize = 157;

#[test]
fn inf_first() {
    let input = inputs(N);

    let result = input.par().filter(|x| *x == &(N / 2).to_string()).first();
    assert_eq!(result, Some(&(N / 2).to_string()));

    let result = input.par().filter(|x| x.as_str() == "x").first();
    assert_eq!(result, None);

    // empty
    let input = inputs(0);
    let result = input.par().filter(|x| *x == &(N / 2).to_string()).first();
    assert_eq!(result, None);
}

#[test]
fn inf_reduce() {
    let input = inputs(N);

    let result = input.par().map(|x| x.len()).reduce(|a, b| a + b);
    assert_eq!(result, input.iter().map(|x| x.len()).reduce(|a, b| a + b));

    // empty
    let input = inputs(0);
    let result = input.par().map(|x| x.len()).reduce(|a, b| a + b);
    assert_eq!(result, None);
}

#[test]
fn inf_collect() {
    let input = inputs(N);
    let result: Vec<String> = input.into_par().filter(|x| x.len() < 2).collect();
    assert_eq!(result, (0..10).map(|x| x.to_string()).collect::<Vec<_>>());

    // empty
    let input = inputs(0);
    let result: Vec<String> = input.into_par().filter(|x| x.len() < 2).collect();
    assert_eq!(result, Vec::<String>::new());
}

#[test]
fn inf_collect_into() {
    let result = vec!["x".to_string()];
    let input = inputs(N);
    let result = input
        .into_par()
        .filter(|x| x.len() < 2)
        .collect_into(result);
    let expected: Vec<_> = ["x", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
        .into_iter()
        .map(|x| x.to_string())
        .collect();
    assert_eq!(result, expected);

    // empty
    let result = vec!["x".to_string()];
    let input = inputs(0);
    let result = input
        .into_par()
        .filter(|x| x.len() < 2)
        .collect_into(result);
    assert_eq!(result, vec!["x".to_string()]);
}

#[test]
fn inf_all() {
    let input = inputs(N);

    let result = input.par().all(|x| x.len() > 0);
    assert_eq!(result, true);

    let result = input.par().all(|x| x.len() == 1);
    assert_eq!(result, false);

    // empty
    let input = inputs(0);

    let result = input.par().all(|x| x.len() > 0);
    assert_eq!(result, true);

    let result = input.par().all(|x| x.len() == 1);
    assert_eq!(result, true);
}

#[test]
fn inf_any() {
    let input = inputs(N);

    let result = input.par().any(|x| x.len() > 1);
    assert_eq!(result, true);

    let result = input.par().any(|x| x.len() == 4);
    assert_eq!(result, false);

    // empty
    let input = inputs(0);

    let result = input.par().any(|x| x.len() > 1);
    assert_eq!(result, false);

    let result = input.par().any(|x| x.len() == 4);
    assert_eq!(result, false);
}

#[test]
fn inf_count() {
    for n in [0, N] {
        let input = inputs(N);

        let result = input.par().filter(|x| x.len() < 2).count();
        assert_eq!(result, input.iter().filter(|x| x.len() < 2).count());

        let result = input.par().filter(|x| x.len() > 4).count();
        assert_eq!(result, input.iter().filter(|x| x.len() > 4).count());
    }
}

#[test]
fn inf_find() {
    for n in [0, N] {
        let input = inputs(n);

        let result = input.par().find(|x| x.len() > 1);
        assert_eq!(result, input.iter().find(|x| x.len() > 1));

        let result = input.par().find(|x| x.len() > 10);
        assert_eq!(result, input.iter().find(|x| x.len() > 10));
    }
}

#[test]
fn inf_find_any() {
    let input = inputs(N);

    let result = input
        .par()
        .iteration_order(IterationOrder::Arbitrary)
        .find(|x| x.len() > 1);
    assert!(result.is_some());

    let result = input
        .par()
        .iteration_order(IterationOrder::Arbitrary)
        .find(|x| x.len() > 10);
    assert_eq!(result, None);

    // empty
    let input = inputs(0);
    let result = input
        .par()
        .iteration_order(IterationOrder::Arbitrary)
        .find(|_| true);
    assert_eq!(result, None);
}

#[test]
fn inf_for_each() {
    for n in [0, N] {
        let input = inputs(N);
        let total_len = AtomicUsize::new(0);
        input
            .par()
            .for_each(|x| _ = total_len.fetch_add(x.len(), Ordering::Relaxed));
        assert_eq!(total_len.into_inner(), input.iter().map(|x| x.len()).sum());
    }
}

#[test]
fn inf_max() {
    for n in [0, N] {
        let input = inputs(n);
        let result = input.par().map(|x| x.len()).max();
        assert_eq!(result, input.iter().map(|x| x.len()).max());
    }
}
