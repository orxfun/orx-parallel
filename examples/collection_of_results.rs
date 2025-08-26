use orx_concurrent_option::ConcurrentOption;
use orx_parallel::*;

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

fn collection_of_results_good_case() {
    for iteration_order in ITERATION_ORDERS {
        let error = ConcurrentOption::none();
        let mut output: Vec<_> = good_input()
            .par()
            .iteration_order(iteration_order)
            .map(|x| match x.parse::<usize>() {
                Ok(x) => Some(x),
                Err(e) => {
                    _ = error.set_some(e);
                    None
                }
            })
            .take_while(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();

        if iteration_order == IterationOrder::Arbitrary {
            output.sort();
        }

        assert!(error.is_none());
        // since no error, whilst will always be true and all results will be collected regardless of order
        assert_eq!(output, (0..N).collect::<Vec<_>>());
    }
}

fn collection_of_results_bad_case() {
    for iteration_order in ITERATION_ORDERS {
        let error = ConcurrentOption::none();
        let mut output: Vec<_> = bad_input()
            .par()
            .iteration_order(iteration_order)
            .map(|x| match x.parse::<usize>() {
                Ok(x) => Some(x),
                Err(e) => {
                    _ = error.set_some(e);
                    None
                }
            })
            .take_while(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();

        if iteration_order == IterationOrder::Arbitrary {
            output.sort();
        }

        assert_eq!(
            error.map(|x| x.to_string()),
            Some("invalid digit found in string".to_string())
        );

        match iteration_order {
            IterationOrder::Ordered => {
                // guaranteed to stop at the first error
                assert_eq!(output, (0..IDX_BAD_INPUT[0]).collect::<Vec<_>>())
            }
            IterationOrder::Arbitrary => {
                // guaranteed to stop at any of the errors and we can take more
                // but everything before the first error are guaranteed to be in
                for i in 0..IDX_BAD_INPUT[0] {
                    assert!(output.contains(&i));
                }
            }
        }
    }
}

fn collect_result() {
    let output: Result<Vec<_>, _> = good_input()
        .par()
        .map(|x| x.parse::<usize>())
        .into_fallible_result()
        .collect();
    assert_eq!(
        output.map_err(|x| x.to_string()),
        Ok((0..N).collect::<Vec<_>>())
    );

    let output: Result<Vec<_>, _> = bad_input()
        .par()
        .map(|x| x.parse::<usize>())
        .into_fallible_result()
        .collect();
    assert_eq!(
        output.map_err(|x| x.to_string()),
        Err("invalid digit found in string".to_string())
    );
}

fn main() {
    collection_of_results_good_case();
    collection_of_results_bad_case();
    collect_result();
}
