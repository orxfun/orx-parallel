use orx_iterable::{IntoCloningIterable, Iterable};
use orx_parallel::*;

fn main() {
    let range = (7..4242).into_iterable();
    let expected: usize = range.iter().sum();

    // all alements are valid, of Ok variant
    let good_input: Vec<_> = range.iter().map(|x| x.to_string()).collect();
    let result = good_input
        .par()
        .map(|x| x.parse::<usize>())
        .into_fallible()
        .reduce(|a, b| a + b);
    assert_eq!(result, Ok(Some(expected)));

    // one element is invalid, of Err variant
    let mut bad_input: Vec<_> = range.iter().map(|x| x.to_string()).collect();
    bad_input.insert(bad_input.len() / 2, "!".to_string());
    let result = bad_input
        .par()
        .map(|x| x.parse::<usize>())
        .into_fallible()
        .reduce(|a, b| a + b);
    assert!(result.is_err()); // computation failed
}
