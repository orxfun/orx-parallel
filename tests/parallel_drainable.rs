use orx_parallel::*;
use std::ops::Range;
use test_case::test_matrix;

#[derive(Clone, Debug)]
struct VecAndRange(Vec<String>, Range<usize>);

impl VecAndRange {
    fn new(n: usize) -> VecAndRange {
        let vec: Vec<_> = (0..n).map(|x| x.to_string()).collect();
        let range = match n % 4 {
            0 => 0..n,
            1 => 0..(n / 2),
            2 => core::cmp::min(n.saturating_sub(1), 3)..n,
            _ => {
                let a = core::cmp::min(n.saturating_sub(1), 3);
                let b = core::cmp::min(a + 10, n);
                a..b
            }
        };
        Self(vec, range)
    }
}

#[test_matrix(
    [0, 1, 2, 3, 71, 72, 73, 74],
    [0, 1],
    [0, 1]
)]
fn parallel_drainable_vec(n: usize, nt: usize, chunk: usize) {
    let vec_and_range = VecAndRange::new(n);
    let VecAndRange(mut vec, range) = vec_and_range;

    let mut vec2 = vec.clone();
    let drained: Vec<_> = vec2.drain(range.clone()).collect();
    let expected = (vec2, drained);

    let drained: Vec<_> = vec
        .par_drain(range)
        .num_threads(nt)
        .chunk_size(chunk)
        .collect();
    let result = (vec, drained);

    assert_eq!(result, expected);
}
