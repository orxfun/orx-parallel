use orx_concurrent_option::ConcurrentOption;
use orx_parallel::*;

const N: usize = 10_000;
const IDX_BAD_INPUT: [usize; 4] = [1900, 4156, 6777, 5663];
const ITERATION_ORDERS: [IterationOrder; 2] = [IterationOrder::Ordered, IterationOrder::Arbitrary];

fn collection_of_results_good_case() {
    for iteration_order in ITERATION_ORDERS {
        let error = ConcurrentOption::none();
        let good_input: Vec<_> = (0..N).map(|x| x.to_string()).collect();
        let mut output: Vec<_> = good_input
            .par()
            .iteration_order(iteration_order)
            .map(|x| match x.parse::<usize>() {
                Ok(x) => Some(x),
                Err(e) => {
                    _ = error.set_some(e);
                    None
                }
            })
            .whilst(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();

        if iteration_order == IterationOrder::Arbitrary {
            output.sort();
        }

        assert!(error.is_none());
        assert_eq!(output, (0..N).collect::<Vec<_>>());
    }
}

fn collection_of_results_bad_case() {
    for iteration_order in ITERATION_ORDERS {
        let error = ConcurrentOption::none();
        let bad_input: Vec<_> = (0..N)
            .map(|x| match IDX_BAD_INPUT.contains(&x) {
                true => format!("{x}!"),
                false => x.to_string(),
            })
            .collect();
        let mut output: Vec<_> = bad_input
            .par()
            .iteration_order(IterationOrder::Arbitrary)
            .map(|x| match x.parse::<usize>() {
                Ok(x) => Some(x),
                Err(e) => {
                    _ = error.set_some(e);
                    None
                }
            })
            .whilst(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();

        if iteration_order == IterationOrder::Arbitrary {
            output.sort();
        }

        assert_eq!(
            error.map(|x| x.to_string()),
            Some("invalid digit found in string".to_string())
        );
        for i in 0..IDX_BAD_INPUT[0] {
            assert!(output.contains(&i));
        }
    }
}

fn main() {
    // collection_of_results_good_case();
    collection_of_results_bad_case();
}
