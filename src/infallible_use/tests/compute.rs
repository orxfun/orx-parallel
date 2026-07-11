use crate::infallible_use::tests::utils::inputs;
use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::string::{String, ToString};

const N: [usize; 2] = [0, 157];

#[test]
fn inf_use_first() {
    for n in N {
        let input = inputs(n);

        let result = input
            .par()
            .use_new(|_| ())
            .filter(|_, x| *x == &(n / 2).to_string())
            .first();
        assert_eq!(
            result,
            input.iter().filter(|x| *x == &(n / 2).to_string()).next()
        );

        let result = input
            .par()
            .use_new(|_| ())
            .filter(|_, x| x.as_str() == "x")
            .first();
        assert_eq!(result, input.iter().filter(|x| x.as_str() == "x").next());
    }
}

#[test]
fn inf_use_reduce() {
    for n in N {
        let input = inputs(n);

        let result = input
            .par()
            .use_new(|_| ())
            .map(|_, x| x.len())
            .reduce(|_, a, b| a + b);
        assert_eq!(result, input.iter().map(|x| x.len()).reduce(|a, b| a + b));
    }
}

#[test]
fn inf_use_fold() {
    for n in N {
        let input = inputs(n);

        let mut expected = String::new();
        input
            .iter()
            .filter(|x| x.len() < 2)
            .for_each(|x| expected.push_str(x));
        let mut expected: Vec<_> = expected.chars().collect();
        expected.sort();

        let par = input
            .clone()
            .into_par()
            .num_threads(4)
            .use_new(|_| ())
            .filter(|_, x| x.len() < 2);
        let result = par.fold(String::new, |_, s, x| s.push_str(&x));
        assert!(result.len() <= 4);
        let result = result
            .into_iter()
            .reduce(|mut a: String, b: String| {
                a.push_str(&b);
                a
            })
            .unwrap_or_default();
        let mut result: Vec<_> = result.chars().collect();
        result.sort();

        assert_eq!(&result, &expected);
    }
}

#[test]
fn inf_use_collect() {
    for n in N {
        let input = inputs(n);
        let result: Vec<String> = input
            .clone()
            .into_par()
            .use_new(|_| ())
            .filter(|_, x| x.len() < 2)
            .collect();
        assert_eq!(
            result,
            input
                .into_iter()
                .filter(|x| x.len() < 2)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn inf_use_collect_into() {
    for n in N {
        let input = inputs(n);

        let mut result = vec!["x".to_string()];
        input
            .clone()
            .into_par()
            .use_new(|_| ())
            .filter(|_, x| x.len() < 2)
            .collect_into(&mut result);

        let mut expected = vec!["x".to_string()];
        expected.extend(input.into_iter().filter(|x| x.len() < 2));

        assert_eq!(result, expected);
    }
}

#[test]
fn inf_use_all() {
    for n in N {
        let input = inputs(n);

        let result = input.par().use_new(|_| ()).all(|_, x| !x.is_empty());
        assert_eq!(result, input.iter().all(|x| !x.is_empty()));

        let result = input.par().use_new(|_| ()).all(|_, x| x.len() == 1);
        assert_eq!(result, input.iter().all(|x| x.len() == 1));
    }
}

#[test]
fn inf_use_any() {
    for n in N {
        let input = inputs(n);

        let result = input.par().use_new(|_| ()).any(|_, x| x.len() > 1);
        assert_eq!(result, input.iter().any(|x| x.len() > 1));

        let result = input.par().use_new(|_| ()).any(|_, x| x.len() == 4);
        assert_eq!(result, input.iter().any(|x| x.len() == 4));
    }
}

#[test]
fn inf_use_count() {
    for n in N {
        let input = inputs(n);

        let result = input
            .par()
            .use_new(|_| ())
            .filter(|_, x| x.len() < 2)
            .count();
        assert_eq!(result, input.iter().filter(|x| x.len() < 2).count());

        let result = input
            .par()
            .use_new(|_| ())
            .filter(|_, x| x.len() > 4)
            .count();
        assert_eq!(result, input.iter().filter(|x| x.len() > 4).count());
    }
}

#[test]
fn inf_use_find() {
    for n in N {
        let input = inputs(n);

        let result = input
            .clone()
            .into_par()
            .use_new(|_| ())
            .find(|_, x| x.len() > 1);
        assert_eq!(result, input.iter().find(|x| x.len() > 1).cloned());

        let result = input
            .clone()
            .into_par()
            .use_new(|_| ())
            .find(|_, x| x.len() > 10);
        assert_eq!(result, input.iter().find(|x| x.len() > 10).cloned());
    }
}

#[test]
fn inf_use_find_any() {
    let input = inputs(N[1]);

    let result = input
        .clone()
        .into_par()
        .use_new(|_| ())
        .iteration_order(IterationOrder::Arbitrary)
        .find(|_, x| x.len() > 1);
    assert!(result.is_some());

    let result = input
        .into_par()
        .use_new(|_| ())
        .iteration_order(IterationOrder::Arbitrary)
        .find(|_, x| x.len() > 10);
    assert_eq!(result, None);

    // empty
    let input = inputs(0);
    let result = input
        .into_par()
        .use_new(|_| ())
        .iteration_order(IterationOrder::Arbitrary)
        .find(|_, _| true);
    assert_eq!(result, None);
}

#[test]
fn inf_use_for_each() {
    for n in N {
        let input = inputs(n);
        let total_len = AtomicUsize::new(0);
        input
            .par()
            .use_new(|_| ())
            .for_each(|_, x| _ = total_len.fetch_add(x.len(), Ordering::Relaxed));
        assert_eq!(total_len.into_inner(), input.iter().map(|x| x.len()).sum());
    }
}

#[test]
fn inf_use_max() {
    for n in N {
        let input = inputs(n);
        let result = input.par().use_new(|_| ()).map(|_, x| x.len()).max();
        assert_eq!(result, input.iter().map(|x| x.len()).max());
    }
}

#[test]
fn inf_use_max_by() {
    for n in N {
        let input = inputs(n);
        let result = input
            .par()
            .use_new(|_| ())
            .map(|_, x| x.len())
            .max_by(|_, a, b| a.cmp(&b));
        assert_eq!(
            result,
            input.iter().map(|x| x.len()).max_by(|a, b| a.cmp(&b))
        );
    }
}

#[test]
fn inf_use_max_by_key() {
    for n in N {
        let input = inputs(n);
        let result = input
            .par()
            .use_new(|_| ())
            .max_by_key(|_, x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(
            result,
            input
                .iter()
                .max_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap())
        );
    }
}

#[test]
fn inf_use_min() {
    for n in N {
        let input = inputs(n);
        let result = input.par().use_new(|_| ()).map(|_, x| x.len()).min();
        assert_eq!(result, input.iter().map(|x| x.len()).min());
    }
}

#[test]
fn inf_use_min_by() {
    for n in N {
        let input = inputs(n);
        let result = input
            .par()
            .use_new(|_| ())
            .map(|_, x| x.len())
            .min_by(|_, a, b| a.cmp(&b));
        assert_eq!(
            result,
            input.iter().map(|x| x.len()).min_by(|a, b| a.cmp(&b))
        );
    }
}

#[test]
fn inf_use_min_by_key() {
    for n in N {
        let input = inputs(n);
        let result = input
            .par()
            .use_new(|_| ())
            .min_by_key(|_, x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(
            result,
            input
                .iter()
                .min_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap())
        );
    }
}

#[test]
fn inf_use_sum() {
    for n in N {
        let input = inputs(n);

        let result = input
            .par()
            .use_new(|_| ())
            .filter(|_, x| x.len() > 1)
            .map(|_, x| x.len())
            .sum();
        assert_eq!(
            result,
            input.iter().filter(|x| x.len() > 1).map(|x| x.len()).sum()
        );
    }
}
