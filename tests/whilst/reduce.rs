use crate::fibonacci;
use criterion::black_box;
use orx_parallel::*;
use test_case::test_case;

#[test_case(511, 0, 0, 333, 332*331/2)]
#[test_case(511, 0, 0, 1000, 510*509/2)]
#[test_case(511, 0, 0, 222, 221*220/2)]
#[test_case(511, 0, 0, 171, 170*169/2)]
fn par(n: usize, nt: usize, c: usize, stop_at: usize, expected_min: usize) {
    let input = 0..n;
    let output = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            *x != stop_at
        })
        .reduce(|x, y| x + y);

    assert!(output.unwrap() >= expected_min);
}

#[test_case(511, 0, 0, 333, 332*331/2)]
#[test_case(511, 0, 0, 1000, 510*509/2)]
#[test_case(511, 0, 0, 222, 221*220/2)]
#[test_case(511, 0, 0, 171, 170*169/2)]
fn www_map(n: usize, nt: usize, c: usize, stop_at: usize, expected_min: usize) {
    let input = 0..n;
    let output = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .map(|x| x.to_string())
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            let x: usize = x.parse().unwrap();
            x != stop_at
        })
        .map(|x| x.parse::<usize>().unwrap())
        .reduce(|x, y| x + y);

    assert!(output.unwrap() >= expected_min);
}

#[test_case(511, 0, 0, 333, 332*331/2)]
#[test_case(511, 0, 0, 1000, 510*509/2)]
#[test_case(511, 0, 0, 222, 221*220/2)]
#[test_case(511, 0, 0, 171, 170*169/2)]
fn filter(n: usize, nt: usize, c: usize, stop_at: usize, expected_min: usize) {
    let input = 0..n;
    let output = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .filter(|x| x != &42)
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            *x != stop_at
        })
        .reduce(|x, y| x + y);

    assert!(output.unwrap() >= expected_min - 42);
}

#[test_case(511, 0, 0, 333, 332*331/2)]
#[test_case(511, 0, 0, 1000, 510*509/2)]
#[test_case(511, 0, 0, 222, 221*220/2)]
#[test_case(511, 0, 0, 171, 170*169/2)]
fn flatmap(n: usize, nt: usize, c: usize, stop_at: usize, expected_min: usize) {
    let input = 0..n;
    let output = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .flat_map(|x| [x, x, x])
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            *x != stop_at
        })
        .reduce(|x, y| x + y);

    assert!(output.unwrap() >= expected_min * 3);
}
