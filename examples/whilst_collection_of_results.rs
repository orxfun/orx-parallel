use orx_concurrent_option::ConcurrentOption;
use orx_parallel::*;

fn collection_of_results_good_case() {
    let error = ConcurrentOption::none();
    let n = 1000;
    let good_input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output: Vec<_> = good_input
        .par()
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

    assert!(error.is_none());
    assert_eq!(output, (0..n).collect::<Vec<_>>());
}

fn collection_of_results_bad_case() {
    let error = ConcurrentOption::none();
    let n = 1000;
    let bad_input: Vec<_> = (0..n)
        .map(|x| match [100, 456, 777].contains(&x) {
            true => format!("{x}!"),
            false => x.to_string(),
        })
        .collect();
    let output: Vec<_> = bad_input
        .par()
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

    assert_eq!(
        error.map(|x| x.to_string()),
        Some("invalid digit found in string".to_string())
    );
    assert_eq!(output, (0..100).collect::<Vec<_>>());
}

fn main() {
    collection_of_results_good_case();
    collection_of_results_bad_case();
}
