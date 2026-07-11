use crate::option::tests::utils::inputs;
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
fn opt_first() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .filter(|x| *x == (n / 2).to_string())
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
            .filter(|x| x.as_str() == "x")
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
            .filter(|x| x.as_str() == "x")
            .first();
        assert_eq!(result, None);
    }
}

#[test]
fn opt_reduce() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .map(|x| x.len())
            .reduce(|a, b| a + b);
        assert_eq!(
            result,
            Some(input.iter().map(|x| x.len()).reduce(|a, b| a + b))
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .map(|x| x.len())
            .reduce(|a, b| a + b);
        assert_eq!(result, None);
    }
}

#[test]
fn opt_collect() {
    for n in N {
        let input = inputs(n);
        let result: Option<Vec<String>> = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .filter(|x| x.len() < 2)
            .collect();
        assert_eq!(
            result,
            Some(
                input
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
            .filter(|x| x.len() < 2)
            .collect();
        assert_eq!(result, None);
    }
}

#[test]
fn opt_collect_into() {
    for n in N {
        let input = inputs(n);

        let mut result = vec!["x".to_string()];
        let ok = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .filter(|x| x.len() < 2)
            .collect_into(&mut result);

        let mut expected = vec!["x".to_string()];
        expected.extend(input.into_iter().filter(|x| x.len() < 2));

        assert_eq!(ok, Some(()));
        assert_eq!(result, expected);

        let input = inputs(core::cmp::max(100, n));
        let mut result = vec!["x".to_string()];
        let err = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .filter(|x| x.len() < 2)
            .collect_into(&mut result);
        assert_eq!(err, None);
    }
}

#[test]
fn opt_all() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .all(|x| !x.is_empty());
        assert_eq!(result, Some(input.iter().all(|x| !x.is_empty())));

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .all(|x| x.len() == 1);
        assert_eq!(result, Some(input.iter().all(|x| x.len() == 1)));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .all(|x| !x.is_empty());
        assert_eq!(result, None);
    }
}

#[test]
fn opt_any() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .any(|x| x.len() > 1);
        assert_eq!(result, Some(input.iter().any(|x| x.len() > 1)));

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .any(|x| x.len() == 4);
        assert_eq!(result, Some(input.iter().any(|x| x.len() == 4)));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .any(|x| x.parse::<usize>().unwrap() > 60);
        assert_eq!(result, None);
    }
}

#[test]
fn opt_count() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .filter(|x| x.len() < 2)
            .count();
        assert_eq!(result, Some(input.iter().filter(|x| x.len() < 2).count()));

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .filter(|x| x.len() > 4)
            .count();
        assert_eq!(result, Some(input.iter().filter(|x| x.len() > 4).count()));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .filter(|x| x.len() < 2)
            .count();
        assert_eq!(result, None);
    }
}

#[test]
fn opt_find() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .find(|x| x.len() > 1);
        assert_eq!(result, Some(input.iter().find(|x| x.len() > 1).cloned()));

        let result = input
            .clone()
            .into_par()
            .map(Some)
            .into_optional()
            .find(|x| x.len() > 10);
        assert_eq!(result, Some(input.iter().find(|x| x.len() > 10).cloned()));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .find(|x| x.len() > 10);
        assert_eq!(result, None);
    }
}

#[test]
fn opt_find_any() {
    let input = inputs(N[1]);

    let result = input
        .clone()
        .into_par()
        .map(Some)
        .into_optional()
        .iteration_order(IterationOrder::Arbitrary)
        .find(|x| x.len() > 1);
    assert!(matches!(result, Some(Some(_))));

    let result = input
        .clone()
        .into_par()
        .map(Some)
        .into_optional()
        .iteration_order(IterationOrder::Arbitrary)
        .find(|x| x.len() > 10);
    assert_eq!(result, Some(None));

    // empty
    let input = inputs(0);
    let result = input
        .into_par()
        .map(Some)
        .into_optional()
        .iteration_order(IterationOrder::Arbitrary)
        .find(|_| true);
    assert_eq!(result, Some(None));
}

#[test]
fn opt_for_each() {
    for n in N {
        let input = inputs(n);
        let total_len_ok = AtomicUsize::new(0);
        let result = input
            .par()
            .map(Some)
            .into_optional()
            .for_each(|x| _ = total_len_ok.fetch_add(x.len(), Ordering::Relaxed));
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
            .for_each(|x| _ = total_len_err.fetch_add(x.len(), Ordering::Relaxed));
        assert_eq!(result, None);
    }
}

#[test]
fn opt_max() {
    for n in N {
        let input = inputs(n);
        let result = input.par().map(Some).into_optional().map(|x| x.len()).max();
        assert_eq!(result, Some(input.iter().map(|x| x.len()).max()));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .map(|x| x.len())
            .max();
        assert_eq!(result, None);
    }
}

#[test]
fn opt_max_by() {
    for n in N {
        let input = inputs(n);
        let result = input
            .par()
            .map(Some)
            .into_optional()
            .map(|x| x.len())
            .max_by(|a, b| a.cmp(b));
        assert_eq!(
            result,
            Some(input.iter().map(|x| x.len()).max_by(|a, b| a.cmp(b)))
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .map(|x| x.len())
            .max_by(|a, b| a.cmp(b));
        assert_eq!(result, None);
    }
}

#[test]
fn opt_max_by_key() {
    for n in N {
        let input = inputs(n);
        let result = input
            .par()
            .map(Some)
            .into_optional()
            .max_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(
            result,
            Some(
                input
                    .iter()
                    .max_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap())
            )
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .max_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(result, None);
    }
}

#[test]
fn opt_min() {
    for n in N {
        let input = inputs(n);
        let result = input.par().map(Some).into_optional().map(|x| x.len()).min();
        assert_eq!(result, Some(input.iter().map(|x| x.len()).min()));

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .map(|x| x.len())
            .min();
        assert_eq!(result, None);
    }
}

#[test]
fn opt_min_by() {
    for n in N {
        let input = inputs(n);
        let result = input
            .par()
            .map(Some)
            .into_optional()
            .map(|x| x.len())
            .min_by(|a, b| a.cmp(b));
        assert_eq!(
            result,
            Some(input.iter().map(|x| x.len()).min_by(|a, b| a.cmp(b)))
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .map(|x| x.len())
            .min_by(|a, b| a.cmp(b));
        assert_eq!(result, None);
    }
}

#[test]
fn opt_min_by_key() {
    for n in N {
        let input = inputs(n);
        let result = input
            .par()
            .map(Some)
            .into_optional()
            .min_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(
            result,
            Some(
                input
                    .iter()
                    .min_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap())
            )
        );

        let input = inputs(core::cmp::max(100, n));
        let result = input
            .into_par()
            .map(some_or_none_at_fifty)
            .into_optional()
            .min_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(result, None);
    }
}

#[test]
fn opt_sum() {
    for n in N {
        let input = inputs(n);

        let result = input
            .par()
            .map(Some)
            .into_optional()
            .filter(|x| x.len() > 1)
            .map(|x| x.len())
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
            .filter(|x| x.len() > 1)
            .map(|x| x.len())
            .sum();
        assert_eq!(result, None);
    }
}
