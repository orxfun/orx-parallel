use super::utils::{make_u_filter, make_u_map};
use crate::{test_utils::*, *};
use alloc::vec::Vec;
use std::string::ToString;
use test_case::test_matrix;

fn input<O: FromIterator<usize>>(n: usize) -> O {
    (0..n).collect()
}

#[test_matrix(N, NT, CHUNK)]
fn copied_cloned_empty(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input: Vec<_> = input::<Vec<_>>(n);
        let expected: usize = input.iter().copied().sum();
        let par = || {
            input
                .par()
                .num_threads(nt)
                .chunk_size(chunk)
                .using_clone("XyZw".to_string())
        };

        let output = par().copied().sum();
        assert_eq!(output, expected);

        let output = par().cloned().sum();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn copied_cloned_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let map = |x: usize| x + 1;
        let expected: usize = input.iter().copied().map(&map).sum();
        let par = || {
            input
                .par()
                .num_threads(nt)
                .chunk_size(chunk)
                .using_clone("XyZw".to_string())
        };

        let output = par().copied().map(make_u_map(map)).sum();
        assert_eq!(output, expected);

        let output = par().cloned().map(make_u_map(map)).sum();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn copied_cloned_xap_flat_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let flat_map = |x: usize| [x, x + 1, x + 2];
        let expected: usize = input.iter().copied().flat_map(&flat_map).sum();
        let par = || {
            input
                .par()
                .num_threads(nt)
                .chunk_size(chunk)
                .using_clone("XyZw".to_string())
        };

        let output = par().copied().flat_map(make_u_map(flat_map)).sum();
        assert_eq!(output, expected);

        let output = par().cloned().flat_map(make_u_map(flat_map)).sum();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn copied_cloned_xap_filter_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let filter_map = |x: usize| (x % 3 != 0).then(|| x + 1);
        let expected: usize = input.iter().copied().filter_map(&filter_map).sum();
        let par = || {
            input
                .par()
                .num_threads(nt)
                .chunk_size(chunk)
                .using_clone("XyZw".to_string())
        };

        let output = par().copied().filter_map(make_u_map(filter_map)).sum();
        assert_eq!(output, expected);

        let output = par().cloned().filter_map(make_u_map(filter_map)).sum();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn copied_cloned_xap_filter_xap(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let filter_map = |x: usize| (x % 3 != 0).then(|| x + 1);
        let filter = |x: &usize| x % 3 != 1;
        let flat_map = |x: usize| [x, x + 1, x + 2];
        let expected: usize = input
            .iter()
            .copied()
            .filter_map(&filter_map)
            .filter(&filter)
            .flat_map(&flat_map)
            .sum();
        let par = || {
            input
                .par()
                .num_threads(nt)
                .chunk_size(chunk)
                .using_clone("XyZw".to_string())
        };

        let output = par()
            .copied()
            .filter_map(make_u_map(filter_map))
            .filter(make_u_filter(&filter))
            .flat_map(make_u_map(flat_map))
            .sum();
        assert_eq!(output, expected);

        let output = par()
            .cloned()
            .filter_map(make_u_map(filter_map))
            .filter(make_u_filter(&filter))
            .flat_map(make_u_map(flat_map))
            .sum();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test]
fn small_example() {
    let n = 10;
    let nt = 1;
    let chunk = 1;

    let input = input::<Vec<_>>(n);
    let filter_map = |x: usize| (x % 3 != 0).then(|| x + 1);
    let filter = |x: &usize| x % 3 != 1;
    let flat_map = |x: usize| [x, x + 1, x + 2];
    let expected: usize = input
        .iter()
        .copied()
        .filter_map(&filter_map)
        .filter(&filter)
        .flat_map(&flat_map)
        .sum();
    let par = || {
        input
            .par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string())
    };

    let output = par()
        .copied()
        .filter_map(make_u_map(filter_map))
        .filter(make_u_filter(&filter))
        .flat_map(make_u_map(flat_map))
        .sum();
    assert_eq!(output, expected);

    let output = par()
        .cloned()
        .filter_map(make_u_map(filter_map))
        .filter(make_u_filter(&filter))
        .flat_map(make_u_map(flat_map))
        .sum();
    assert_eq!(output, expected);
}
