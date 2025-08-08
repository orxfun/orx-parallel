use criterion::black_box;
use orx_parallel::*;
use std::collections::HashMap;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Data {
    name: String,
    number: usize,
}

fn to_output(idx: usize) -> Data {
    let name = idx.to_string();
    let number = idx;
    Data { name, number }
}

fn filter(data: &&mut Data) -> bool {
    !data.name.starts_with('3')
}

fn update(data: &mut Data) {
    for _ in 0..50 {
        let increment = black_box(1);
        data.number += increment
    }
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64])
]
fn mut_iter(n: usize, nt: usize, chunk: usize) {
    let input: HashMap<usize, Data> = (0..n).map(|i| (i, to_output(i))).collect();
    let mut expected = input.clone();
    expected.values_mut().filter(filter).for_each(update);
    let expected = expected;

    let mut input = input.clone();
    input
        .values_mut()
        .iter_into_par()
        .num_threads(nt)
        .chunk_size(chunk)
        .filter(filter)
        .for_each(update);
    assert_eq!(expected, input);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64])
]
fn mut_slice(n: usize, nt: usize, chunk: usize) {
    let input: Vec<Data> = (0..n).map(|i| to_output(i)).collect();
    let mut expected = input.clone();
    expected.iter_mut().filter(filter).for_each(update);
    let expected = expected;

    let mut input = input.clone();
    input
        .par_mut()
        .num_threads(nt)
        .chunk_size(chunk)
        .filter(filter)
        .for_each(update);
    assert_eq!(expected, input);
}
