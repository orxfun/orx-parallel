use crate::{test_utils::test_n_nt_chunk, *};
use test_case::test_matrix;

#[cfg(not(miri))]
const N: &[usize] = &[8025, 42735];
#[cfg(not(miri))]
const NT: &[usize] = &[1, 2, 4];
#[cfg(not(miri))]
const CHUNK: &[usize] = &[1, 64, 1024];

#[cfg(miri)]
const N: &[usize] = &[37, 125];
#[cfg(miri)]
const NT: &[usize] = &[3];
#[cfg(miri)]
const CHUNK: &[usize] = &[1, 64];

fn input<O: FromIterator<String>>(n: usize) -> O {
    let elem = |x: usize| (x + 10).to_string();
    (0..n).map(elem).collect()
}

#[test_matrix(N, NT, CHUNK)]
fn sum_empty(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input: Vec<_> = input::<Vec<_>>(n).iter().map(|x| x.len()).collect();
        let expected: usize = input.iter().sum();
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output = par.sum();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn sum_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let map = |x: String| x.len();
        let expected: usize = input.clone().into_iter().map(&map).sum();
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let par = par.map(map);
        let output = par.sum();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn sum_xap_flat_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let flat_map = |x: String| x.chars().map(|x| x.to_string().len()).collect::<Vec<_>>();
        let expected: usize = input.clone().into_iter().flat_map(&flat_map).sum();
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let par = par.flat_map(flat_map);
        let output = par.sum();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn sum_xap_filter_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let filter_map = |x: String| x.starts_with('3').then(|| x.len());
        let expected: usize = input.clone().into_iter().filter_map(&filter_map).sum();
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let par = par.filter_map(filter_map);
        let output = par.sum();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn sum_xap_filter_xap(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let filter_map = |x: String| x.starts_with('3').then_some(x);
        let filter = |x: &String| x.ends_with('3');
        let flat_map = |x: String| x.chars().map(|x| x.to_string().len()).collect::<Vec<_>>();
        let expected = input
            .clone()
            .into_iter()
            .filter_map(&filter_map)
            .filter(&filter)
            .flat_map(&flat_map)
            .sum();
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let par = par.filter_map(filter_map).filter(filter).flat_map(flat_map);
        let output = par.sum();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}
