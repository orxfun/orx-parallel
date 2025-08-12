use crate::fibonacci;
use criterion::black_box;
use orx_parallel::*;
use test_case::test_case;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

#[test_case(0, 0, 0, 0, "0")]
#[test_case(512, 4, 0, 3, "22")]
#[test_case(1024, 3, 0, 3, "84")]
#[test_case(1024, 4, 1, 3, "84")]
#[test_case(1024, 2, 0, 2, "5")]
fn w_par(n: usize, nt: usize, c: usize, until_num_digits: usize, until_digits: &str) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output: Vec<_> = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .whilst(|x| {
            let _fib = black_box(fibonacci(42));
            x.len() != until_num_digits || !x.starts_with(&until_digits)
        })
        .collect();
    let expected: Vec<_> = (0..n)
        .map(|x| x.to_string())
        .take_while(|x| x.len() != until_num_digits || !x.starts_with(&until_digits))
        .collect();

    assert_eq!(output, expected);
}
