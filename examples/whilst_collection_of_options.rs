use orx_parallel::*;
use std::sync::atomic::{AtomicBool, Ordering};

const N: usize = 10_000;
const IDX_BAD_INPUT: [usize; 4] = [1900, 4156, 6777, 5663];
const ITERATION_ORDERS: [IterationOrder; 2] = [IterationOrder::Ordered, IterationOrder::Arbitrary];

fn good_input() -> Vec<String> {
    (0..N).map(|x| x.to_string()).collect()
}

fn bad_input() -> Vec<String> {
    (0..N)
        .map(|x| match IDX_BAD_INPUT.contains(&x) {
            true => format!("{x}!"),
            false => x.to_string(),
        })
        .collect()
}

fn collection_of_options_good_case() {
    for iteration_order in ITERATION_ORDERS {
        let has_none = AtomicBool::new(false);
        let mut output: Vec<_> = good_input()
            .par()
            .iteration_order(iteration_order)
            .map(|x| match x.parse::<usize>().ok() {
                Some(x) => Some(x),
                None => {
                    _ = has_none.fetch_or(true, Ordering::Relaxed);
                    None
                }
            })
            .whilst(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();

        if iteration_order == IterationOrder::Arbitrary {
            output.sort();
        }

        assert!(!has_none.load(Ordering::Relaxed));
        // since no None's, whilst will always be true and all results will be collected regardless of order
        assert_eq!(output, (0..N).collect::<Vec<_>>());
    }
}

fn collection_of_options_bad_case() {
    for iteration_order in ITERATION_ORDERS {
        let has_none = AtomicBool::new(false);
        let mut output: Vec<_> = bad_input()
            .par()
            .iteration_order(iteration_order)
            .map(|x| match x.parse::<usize>().ok() {
                Some(x) => Some(x),
                None => {
                    _ = has_none.fetch_or(true, Ordering::Relaxed);
                    None
                }
            })
            .whilst(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();

        if iteration_order == IterationOrder::Arbitrary {
            output.sort();
        }

        assert!(has_none.load(Ordering::Relaxed));

        match iteration_order {
            IterationOrder::Ordered => {
                // guaranteed to stop at the first error
                assert_eq!(output, (0..IDX_BAD_INPUT[0]).collect::<Vec<_>>())
            }
            IterationOrder::Arbitrary => {
                // guaranteed to stop at any of the None's and we can take more
                // but everything before the first None are guaranteed to be in
                for i in 0..IDX_BAD_INPUT[0] {
                    assert!(output.contains(&i));
                }
            }
        }
    }
}

fn collect_option() {
    let output: Option<Vec<_>> = good_input()
        .par()
        .map(|x| x.parse::<usize>().ok())
        .collect_option();
    assert_eq!(output, Some((0..N).collect::<Vec<_>>()));

    let output: Option<Vec<_>> = bad_input()
        .par()
        .map(|x| x.parse::<usize>().ok())
        .collect_option();
    assert_eq!(output, None);
}

fn main() {
    collection_of_options_good_case();
    collection_of_options_bad_case();
    collect_option();
}
