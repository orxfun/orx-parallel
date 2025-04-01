use crate::*;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [8025, 42735];

fn input<O: FromIterator<usize>>(n: usize) -> O {
    (0..n).collect()
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn copied_cloned_empty(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = input::<Vec<_>>(n);
    let expected: usize = input.iter().copied().sum();
    let par = || input.par().num_threads(nt).chunk_size(chunk);

    let output = par().copied().sum();
    assert_eq!(output, expected);

    let output = par().cloned().sum();
    assert_eq!(output, expected);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn copied_cloned_map(n: usize, nt: usize, chunk: usize) {
    let input = input::<Vec<_>>(n);
    let map = |x: usize| x + 1;
    let expected: usize = input.iter().copied().map(&map).sum();
    let par = || input.par().num_threads(nt).chunk_size(chunk);

    let output = par().copied().map(map).sum();
    assert_eq!(output, expected);

    let output = par().cloned().map(map).sum();
    assert_eq!(output, expected);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn copied_cloned_xap_flat_map(n: usize, nt: usize, chunk: usize) {
    let input = input::<Vec<_>>(n);
    let flat_map = |x: usize| [x, x + 1, x + 2];
    let expected: usize = input.iter().copied().flat_map(&flat_map).sum();
    let par = || input.par().num_threads(nt).chunk_size(chunk);

    let output = par().copied().flat_map(flat_map).sum();
    assert_eq!(output, expected);

    let output = par().cloned().flat_map(flat_map).sum();
    assert_eq!(output, expected);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn copied_cloned_xap_filter_map(n: usize, nt: usize, chunk: usize) {
    let input = input::<Vec<_>>(n);
    let filter_map = |x: usize| (x % 3 != 0).then(|| x + 1);
    let expected: usize = input.iter().copied().filter_map(&filter_map).sum();
    let par = || input.par().num_threads(nt).chunk_size(chunk);

    let output = par().copied().filter_map(filter_map).sum();
    assert_eq!(output, expected);

    let output = par().cloned().filter_map(filter_map).sum();
    assert_eq!(output, expected);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn copied_cloned_xap_filter_xap(n: usize, nt: usize, chunk: usize) {
    let input = input::<Vec<_>>(n);
    let filter_map = |x: usize| (x % 3 != 0).then(|| x + 1);
    let filter = |x: &usize| (x % 3 != 1);
    let flat_map = |x: usize| [x, x + 1, x + 2];
    let expected = input
        .iter()
        .copied()
        .filter_map(&filter_map)
        .filter(&filter)
        .flat_map(&flat_map)
        .sum();
    let par = || input.par().num_threads(nt).chunk_size(chunk);

    let output = par()
        .copied()
        .filter_map(filter_map)
        .filter(filter)
        .flat_map(flat_map)
        .sum();
    assert_eq!(output, expected);

    let output = par()
        .cloned()
        .filter_map(filter_map)
        .filter(filter)
        .flat_map(flat_map)
        .sum();
    assert_eq!(output, expected);
}
