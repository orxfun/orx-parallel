// use orx_iterable::{IntoCloningIterable, Iterable};
// use orx_parallel::ParCol;

// fn main() {
//     let range = (7..4242).map(|x| 2 * x).into_iterable();
//     let expected: usize = range.iter().sum();

//     // all alements are valid, of Some variant
//     let good_input: Vec<_> = range.iter().collect();
//     let result = good_input
//         .par()
//         .map(|x| x.is_multiple_of(2).then_some(*x))
//         .into_optional()
//         .reduce(|a, b| a + b);
//     assert_eq!(result, Some(Some(expected)));

//     // one element is invalid, of None variant
//     let mut bad_input: Vec<_> = range.iter().collect();
//     bad_input.insert(bad_input.len() / 2, 7);
//     let result = bad_input
//         .par()
//         .map(|x| x.is_multiple_of(2).then_some(*x))
//         .into_optional()
//         .reduce(|a, b| a + b);
//     assert_eq!(result, None); // computation failed
// }
fn main() {}
