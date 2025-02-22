use orx_parallel::*;
use test_case::test_case;

const LEN: usize = 65987;

fn inputs(len: usize) -> Vec<usize> {
    (0..len).map(|i| 1000 + 2 * i).collect()
}

fn sum(a: usize, b: usize) -> usize {
    a + b
}

fn seq<F>(inputs: &[usize], reduce: &F) -> Option<usize>
where
    F: Fn(usize, usize) -> usize + Send + Sync,
{
    inputs.iter().copied().reduce(reduce)
}

#[test_case(1, 1, sum)]
#[test_case(4, 1, sum)]
#[test_case(1, 64, sum)]
#[test_case(4, 64, sum)]
#[test_case(4, 4096, sum)]
#[test_case(4, 65536, sum)]
fn reduce_sum<F>(num_threads: usize, chunk_size: usize, reduce: F)
where
    F: Fn(usize, usize) -> usize + Send + Sync + Clone,
{
    let inputs = inputs(LEN);
    let expected = seq(&inputs, &reduce);

    let result = inputs
        .par()
        .cloned()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .reduce(reduce);
    assert_eq!(result, expected);
}
