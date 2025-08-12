use crate::fibonacci;
use criterion::black_box;
use orx_parallel::*;
use test_case::test_case;

#[test_case(511, 0, 0, &[333], &[333], None)]
#[test_case(511, 0, 0, &[333], &[332], Some(332))]
#[test_case(511, 0, 0, &[222, 333], &[223, 224, 225, 332], None)]
fn o_par(
    n: usize,
    nt: usize,
    c: usize,
    stop_at: &[usize],
    find: &[usize],
    expected: Option<usize>,
) {
    let filter = |x: &String| find.contains(&x.parse().unwrap());

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .whilst(|x| {
            let _fib = black_box(fibonacci(42));
            let num: usize = x.parse().unwrap();
            !stop_at.contains(&num)
        })
        .filter(filter)
        .first()
        // .find(filter)
        .map(|x| x.parse().unwrap());

    assert_eq!(output, expected);
}
