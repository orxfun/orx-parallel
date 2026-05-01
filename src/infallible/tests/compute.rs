use crate::infallible::tests::utils::inputs;
use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::string::{String, ToString};

const N: [usize; 2] = [0, 157];

#[test]
fn inf_first() {
    for n in N {
        let input = inputs(n);

        let result = input.par().filter(|x| *x == &(n / 2).to_string()).first();
        assert_eq!(
            result,
            input.iter().filter(|x| *x == &(n / 2).to_string()).next()
        );

        let result = input.par().filter(|x| x.as_str() == "x").first();
        assert_eq!(result, input.iter().filter(|x| x.as_str() == "x").next());
    }
}

#[test]
fn inf_reduce() {
    for n in N {
        let input = inputs(n);

        let result = input.par().map(|x| x.len()).reduce(|a, b| a + b);
        assert_eq!(result, input.iter().map(|x| x.len()).reduce(|a, b| a + b));
    }
}

#[test]
fn inf_collect() {
    for n in N {
        let input = inputs(n);
        let result: Vec<String> = input.clone().into_par().filter(|x| x.len() < 2).collect();
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
fn inf_collect_into() {
    for n in N {
        let input = inputs(n);

        let result = vec!["x".to_string()];
        let result = input
            .clone()
            .into_par()
            .filter(|x| x.len() < 2)
            .collect_into(result);

        let mut expected = vec!["x".to_string()];
        expected.extend(input.into_iter().filter(|x| x.len() < 2));

        assert_eq!(result, expected);
    }
}

#[test]
fn inf_all() {
    for n in N {
        let input = inputs(n);

        let result = input.par().all(|x| x.len() > 0);
        assert_eq!(result, input.iter().all(|x| x.len() > 0));

        let result = input.par().all(|x| x.len() == 1);
        assert_eq!(result, input.iter().all(|x| x.len() == 1));
    }
}

#[test]
fn inf_any() {
    for n in N {
        let input = inputs(n);

        let result = input.par().any(|x| x.len() > 1);
        assert_eq!(result, input.iter().any(|x| x.len() > 1));

        let result = input.par().any(|x| x.len() == 4);
        assert_eq!(result, input.iter().any(|x| x.len() == 4));
    }
}

#[test]
fn inf_count() {
    for n in N {
        let input = inputs(n);

        let result = input.par().filter(|x| x.len() < 2).count();
        assert_eq!(result, input.iter().filter(|x| x.len() < 2).count());

        let result = input.par().filter(|x| x.len() > 4).count();
        assert_eq!(result, input.iter().filter(|x| x.len() > 4).count());
    }
}

#[test]
fn inf_find() {
    for n in N {
        let input = inputs(n);

        let result = input.par().find(|x| x.len() > 1);
        assert_eq!(result, input.iter().find(|x| x.len() > 1));

        let result = input.par().find(|x| x.len() > 10);
        assert_eq!(result, input.iter().find(|x| x.len() > 10));
    }
}

#[test]
fn inf_find_any() {
    let input = inputs(N[1]);

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
    for n in N {
        let input = inputs(n);
        let total_len = AtomicUsize::new(0);
        input
            .par()
            .for_each(|x| _ = total_len.fetch_add(x.len(), Ordering::Relaxed));
        assert_eq!(total_len.into_inner(), input.iter().map(|x| x.len()).sum());
    }
}

#[test]
fn inf_max() {
    for n in N {
        let input = inputs(n);
        let result = input.par().map(|x| x.len()).max();
        assert_eq!(result, input.iter().map(|x| x.len()).max());
    }
}

#[test]
fn inf_max_by() {
    for n in N {
        let input = inputs(n);
        let result = input.par().map(|x| x.len()).max_by(|a, b| a.cmp(&b));
        assert_eq!(
            result,
            input.iter().map(|x| x.len()).max_by(|a, b| a.cmp(&b))
        );
    }
}

#[test]
fn inf_max_by_key() {
    for n in N {
        let input = inputs(n);
        let result = input
            .par()
            .max_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(
            result,
            input
                .iter()
                .max_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap())
        );
    }
}

#[test]
fn inf_min() {
    for n in N {
        let input = inputs(n);
        let result = input.par().map(|x| x.len()).min();
        assert_eq!(result, input.iter().map(|x| x.len()).min());
    }
}

#[test]
fn inf_min_by() {
    for n in N {
        let input = inputs(n);
        let result = input.par().map(|x| x.len()).min_by(|a, b| a.cmp(&b));
        assert_eq!(
            result,
            input.iter().map(|x| x.len()).min_by(|a, b| a.cmp(&b))
        );
    }
}

#[test]
fn inf_min_by_key() {
    for n in N {
        let input = inputs(n);
        let result = input
            .par()
            .min_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap());
        assert_eq!(
            result,
            input
                .iter()
                .min_by_key(|x| x.len() * 100 + x.parse::<usize>().unwrap())
        );
    }
}

#[test]
fn inf_sum() {
    for n in N {
        let input = inputs(n);

        let result = input.par().filter(|x| x.len() > 1).map(|x| x.len()).sum();
        assert_eq!(
            result,
            input.iter().filter(|x| x.len() > 1).map(|x| x.len()).sum()
        );
    }
}
