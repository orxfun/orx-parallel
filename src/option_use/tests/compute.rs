use crate::option_use::tests::utils::inputs;
use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::string::{String, ToString};

const N: [usize; 2] = [0, 157];

fn some_or_none_at_fifty(x: String) -> Option<String> {
    match x.as_str() {
        "50" => None,
        _ => Some(x),
    }
}

#[test]
fn opt_use_first() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .filter(|_, x| *x == (n / 2).to_string())
            .first();
        assert_eq!(
            result,
            Some(input.iter().find(|x| *x == &(n / 2).to_string()).cloned())
        );

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .filter(|_, x| x.as_str() == "x")
            .first();
        assert_eq!(
            result,
            Some(input.iter().find(|x| x.as_str() == "x").cloned())
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .filter(|_, x| x.as_str() == "x")
            .first();
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_reduce() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .map(|_, x| x.len())
            .reduce(|_, a, b| a + b);
        assert_eq!(
            result,
            Some(input.iter().map(|x| x.len()).reduce(|a, b| a + b))
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .map(|_, x| x.len())
            .reduce(|_, a, b| a + b);
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_collect() {
    for n in N {
        let input = inputs(n);
        let result: Option<Vec<String>> = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .collect();
        assert_eq!(
            result,
            Some(
                input
                    .clone()
                    .into_iter()
                    .filter(|x| x.len() < 2)
                    .collect::<Vec<_>>()
            )
        );

        let input = inputs(core::cmp::max(100, n));
        let result: Option<Vec<String>> = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .collect();
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_collect_into() {
    for n in N {
        let input = inputs(n);

        let result = vec!["x".to_string()];
        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .collect_into(result);

        let mut expected = vec!["x".to_string()];
        expected.extend(input.into_iter().filter(|x| x.len() < 2));

        assert_eq!(result, Some(expected));

        let input = inputs(core::cmp::max(100, n));
        let result = vec!["x".to_string()];
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .collect_into(result);
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_all() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .all(|_, x| x.len() > 0);
        assert_eq!(result, Some(input.iter().all(|x| x.len() > 0)));

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .all(|_, x| x.len() == 1);
        assert_eq!(result, Some(input.iter().all(|x| x.len() == 1)));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .all(|_, x| x.len() > 0);
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_any() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .any(|_, x| x.len() > 1);
        assert_eq!(result, Some(input.iter().any(|x| x.len() > 1)));

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .any(|_, x| x.len() == 4);
        assert_eq!(result, Some(input.iter().any(|x| x.len() == 4)));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .any(|_, x| x.parse::<usize>().unwrap() > 60);
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_count() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .count();
        assert_eq!(result, Some(input.iter().filter(|x| x.len() < 2).count()));

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .filter(|_, x| x.len() > 4)
            .count();
        assert_eq!(result, Some(input.iter().filter(|x| x.len() > 4).count()));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .filter(|_, x| x.len() < 2)
            .count();
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_find() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .find(|_, x| x.len() > 1);
        assert_eq!(result, Some(input.iter().find(|x| x.len() > 1).cloned()));

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .find(|_, x| x.len() > 10);
        assert_eq!(result, Some(input.iter().find(|x| x.len() > 10).cloned()));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .find(|_, x| x.len() > 10);
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_find_any() {
    let input = inputs(N[1]);

    let result = input
        .clone()
        .into_par()
        .map(Some)
        .into_optional()
        .using_clone(())
        .iteration_order(IterationOrder::Arbitrary)
        .find(|_, x| x.len() > 1);
    assert!(matches!(result, Some(Some(_))));

    let result = input
        .clone()
        .into_par()
        .map(Some)
        .into_optional()
        .using_clone(())
        .iteration_order(IterationOrder::Arbitrary)
        .find(|_, x| x.len() > 10);
    assert_eq!(result, Some(None));

    // empty
    let input = inputs(0);
    let result = input
        .into_par()
        .map(Some)
        .into_optional()
        .using_clone(())
        .iteration_order(IterationOrder::Arbitrary)
        .find(|_, _| true);
    assert_eq!(result, Some(None));
}

#[test]
fn opt_use_for_each() {
    for n in N {
        let input = inputs(n);
        let total_len_ok = AtomicUsize::new(0);
        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .for_each(|_, x| _ = total_len_ok.fetch_add(x.len(), Ordering::Relaxed));
        assert_eq!(result, Some(()));
        assert_eq!(
            total_len_ok.into_inner(),
            input.iter().map(|x| x.len()).sum()
        );

        let input = inputs(core::cmp::max(100, n));
        let total_len_err = AtomicUsize::new(0);
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .for_each(|_, x| _ = total_len_err.fetch_add(x.len(), Ordering::Relaxed));
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_max() {
    for n in N {
        let input = inputs(n);
        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .map(|_, x| x.len())
            .max();
        assert_eq!(result, Some(input.iter().map(|x| x.len()).max()));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .map(|_, x| x.len())
            .max();
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_max_by() {
    for n in N {
        let input = inputs(n);
        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .map(|_, x| x.len())
            .max_by(|_, a, b| a.cmp(b));
        assert_eq!(
            result,
            Some(input.iter().map(|x| x.len()).max_by(|a, b| a.cmp(b)))
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .map(|_, x| x.len())
            .max_by(|_, a, b| a.cmp(b));
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_max_by_key() {
    for n in N {
        let input = inputs(n);
        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .max_by_key(|_, x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(
            result,
            Some(
                input
                    .iter()
                    .max_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap())
                    .cloned()
            )
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .max_by_key(|_, x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_min() {
    for n in N {
        let input = inputs(n);
        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .map(|_, x| x.len())
            .min();
        assert_eq!(result, Some(input.iter().map(|x| x.len()).min()));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .map(|_, x| x.len())
            .min();
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_min_by() {
    for n in N {
        let input = inputs(n);
        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .map(|_, x| x.len())
            .min_by(|_, a, b| a.cmp(b));
        assert_eq!(
            result,
            Some(input.iter().map(|x| x.len()).min_by(|a, b| a.cmp(b)))
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .map(|_, x| x.len())
            .min_by(|_, a, b| a.cmp(b));
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_min_by_key() {
    for n in N {
        let input = inputs(n);
        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .min_by_key(|_, x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(
            result,
            Some(
                input
                    .iter()
                    .min_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap())
                    .cloned()
            )
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .min_by_key(|_, x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(result, None);
    }
}

#[test]
fn opt_use_sum() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .using_clone(())
            .filter(|_, x| x.len() > 1)
            .map(|_, x| x.len())
            .sum();
        assert_eq!(
            result,
            Some(input.iter().filter(|x| x.len() > 1).map(|x| x.len()).sum())
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .using_clone(())
            .filter(|_, x| x.len() > 1)
            .map(|_, x| x.len())
            .sum();
        assert_eq!(result, None);
    }
}
