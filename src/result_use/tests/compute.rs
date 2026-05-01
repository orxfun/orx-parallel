use crate::result_use::tests::utils::inputs;
use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::string::{String, ToString};

const N: [usize; 2] = [0, 157];

fn ok_or_err_at_fifty(x: String) -> Result<String, char> {
    match x.as_str() {
        "50" => Err('x'),
        _ => Ok(x),
    }
}

#[test]
fn res_use_first() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| *x == (n / 2).to_string())
            .first();
        assert_eq!(
            result,
            Ok(input.iter().find(|x| *x == &(n / 2).to_string()).cloned())
        );

        let result = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| x.as_str() == "x")
            .first();
        assert_eq!(
            result,
            Ok(input.iter().find(|x| x.as_str() == "x").cloned())
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| x.as_str() == "x")
            .first();
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_reduce() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .map(|_, x| x.len())
            .reduce(|_, a, b| a + b);
        assert_eq!(
            result,
            Ok(input.iter().map(|x| x.len()).reduce(|a, b| a + b))
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .map(|_, x| x.len())
            .reduce(|_, a, b| a + b);
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_collect() {
    for n in N {
        let input = inputs(n);
        let result: Result<Vec<String>, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .collect();
        assert_eq!(
            result,
            Ok(input
                .into_iter()
                .filter(|x| x.len() < 2)
                .collect::<Vec<_>>())
        );

        let input = inputs(core::cmp::max(100, n));
        let result: Result<Vec<String>, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .collect();
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_collect_into() {
    for n in N {
        let input = inputs(n);

        let mut result = vec!["x".to_string()];
        let ok: Result<(), char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .collect_into(&mut result);

        let mut expected = vec!["x".to_string()];
        expected.extend(input.into_iter().filter(|x| x.len() < 2));

        assert_eq!(ok, Ok(()));
        assert_eq!(result, expected);

        let input = inputs(core::cmp::max(100, n));
        let mut result = vec!["x".to_string()];
        let err: Result<(), char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .collect_into(&mut result);
        assert_eq!(err, Err('x'));
    }
}

#[test]
fn res_use_all() {
    for n in N {
        let input = inputs(n);

        let result: Result<bool, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .all(|_, x| x.len() > 0);
        assert_eq!(result, Ok(input.iter().all(|x| x.len() > 0)));

        let result: Result<bool, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .all(|_, x| x.len() == 1);
        assert_eq!(result, Ok(input.iter().all(|x| x.len() == 1)));

        let input = inputs(core::cmp::max(100, n));
        let result: Result<bool, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .all(|_, x| x.len() > 0);
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_any() {
    for n in N {
        let input = inputs(n);

        let result: Result<bool, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .any(|_, x| x.len() > 1);
        assert_eq!(result, Ok(input.iter().any(|x| x.len() > 1)));

        let result: Result<bool, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .any(|_, x| x.len() == 4);
        assert_eq!(result, Ok(input.iter().any(|x| x.len() == 4)));

        let input = inputs(core::cmp::max(100, n));
        let result: Result<bool, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .any(|_, x| x.parse::<usize>().unwrap() > 60);
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_count() {
    for n in N {
        let input = inputs(n);

        let result: Result<usize, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .count();
        assert_eq!(result, Ok(input.iter().filter(|x| x.len() < 2).count()));

        let result: Result<usize, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| x.len() > 4)
            .count();
        assert_eq!(result, Ok(input.iter().filter(|x| x.len() > 4).count()));

        let input = inputs(core::cmp::max(100, n));
        let result: Result<usize, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .count();
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_find() {
    for n in N {
        let input = inputs(n);

        let result: Result<Option<String>, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .find(|_, x| x.len() > 1);
        assert_eq!(result, Ok(input.iter().find(|x| x.len() > 1).cloned()));

        let result: Result<Option<String>, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .find(|_, x| x.len() > 10);
        assert_eq!(result, Ok(input.iter().find(|x| x.len() > 10).cloned()));

        let input = inputs(core::cmp::max(100, n));
        let result: Result<Option<String>, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .find(|_, x| x.len() > 10);
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_find_any() {
    let input = inputs(N[1]);

    let result: Result<Option<String>, char> = input
        .clone()
        .into_par()
        .map(Ok::<_, char>)
        .into_fallible()
        .using_clone(())
        .iteration_order(IterationOrder::Arbitrary)
        .find(|_, x| x.len() > 1);
    assert!(matches!(result, Ok(Some(_))));

    let result: Result<Option<String>, char> = input
        .clone()
        .into_par()
        .map(Ok::<_, char>)
        .into_fallible()
        .using_clone(())
        .iteration_order(IterationOrder::Arbitrary)
        .find(|_, x| x.len() > 10);
    assert_eq!(result, Ok(None));

    // empty
    let result: Result<Option<String>, char> = inputs(0)
        .into_par()
        .map(Ok::<_, char>)
        .into_fallible()
        .using_clone(())
        .iteration_order(IterationOrder::Arbitrary)
        .find(|_, _| true);
    assert_eq!(result, Ok(None));
}

#[test]
fn res_use_for_each() {
    for n in N {
        let input = inputs(n);
        let total_len_ok = AtomicUsize::new(0);
        let result: Result<(), char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .for_each(|_, x| _ = total_len_ok.fetch_add(x.len(), Ordering::Relaxed));
        assert_eq!(result, Ok(()));
        assert_eq!(
            total_len_ok.into_inner(),
            input.iter().map(|x| x.len()).sum()
        );

        let input = inputs(core::cmp::max(100, n));
        let total_len_err = AtomicUsize::new(0);
        let result: Result<(), char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .for_each(|_, x| _ = total_len_err.fetch_add(x.len(), Ordering::Relaxed));
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_max() {
    for n in N {
        let input = inputs(n);
        let result: Result<Option<usize>, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .map(|_, x| x.len())
            .max();
        assert_eq!(result, Ok(input.iter().map(|x| x.len()).max()));

        let input = inputs(core::cmp::max(100, n));
        let result: Result<Option<usize>, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .map(|_, x| x.len())
            .max();
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_max_by() {
    for n in N {
        let input = inputs(n);
        let result: Result<Option<usize>, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .map(|_, x| x.len())
            .max_by(|_, a, b| a.cmp(b));
        assert_eq!(
            result,
            Ok(input.iter().map(|x| x.len()).max_by(|a, b| a.cmp(b)))
        );

        let input = inputs(core::cmp::max(100, n));
        let result: Result<Option<usize>, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .map(|_, x| x.len())
            .max_by(|_, a, b| a.cmp(b));
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_max_by_key() {
    for n in N {
        let input = inputs(n);
        let result: Result<Option<String>, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .max_by_key(|_, x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(
            result,
            Ok(input
                .iter()
                .max_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap())
                .cloned())
        );

        let input = inputs(core::cmp::max(100, n));
        let result: Result<Option<String>, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .max_by_key(|_, x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_min() {
    for n in N {
        let input = inputs(n);
        let result: Result<Option<usize>, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .map(|_, x| x.len())
            .min();
        assert_eq!(result, Ok(input.iter().map(|x| x.len()).min()));

        let input = inputs(core::cmp::max(100, n));
        let result: Result<Option<usize>, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .map(|_, x| x.len())
            .min();
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_min_by() {
    for n in N {
        let input = inputs(n);
        let result: Result<Option<usize>, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .map(|_, x| x.len())
            .min_by(|_, a, b| a.cmp(b));
        assert_eq!(
            result,
            Ok(input.iter().map(|x| x.len()).min_by(|a, b| a.cmp(b)))
        );

        let input = inputs(core::cmp::max(100, n));
        let result: Result<Option<usize>, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .map(|_, x| x.len())
            .min_by(|_, a, b| a.cmp(b));
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_min_by_key() {
    for n in N {
        let input = inputs(n);
        let result: Result<Option<String>, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .min_by_key(|_, x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(
            result,
            Ok(input
                .iter()
                .min_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap())
                .cloned())
        );

        let input = inputs(core::cmp::max(100, n));
        let result: Result<Option<String>, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .min_by_key(|_, x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(result, Err('x'));
    }
}

#[test]
fn res_use_sum() {
    for n in N {
        let input = inputs(n);

        let result: Result<usize, char> = input
            .clone()
            .into_par()
            .map(Ok::<_, char>)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| x.len() > 1)
            .map(|_, x| x.len())
            .sum();
        assert_eq!(
            result,
            Ok(input.iter().filter(|x| x.len() > 1).map(|x| x.len()).sum())
        );

        let input = inputs(core::cmp::max(100, n));
        let result: Result<usize, char> = input
            .into_par()
            .map(ok_or_err_at_fifty)
            .into_fallible()
            .using_clone(())
            .filter(|_, x| x.len() > 1)
            .map(|_, x| x.len())
            .sum();
        assert_eq!(result, Err('x'));
    }
}
