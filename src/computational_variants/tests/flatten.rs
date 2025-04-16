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

#[test_matrix(N, NT, CHUNK)]
fn flatten_empty(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || {
            (0..n)
                .map(|i| [i.to_string(), (i + 1).to_string()])
                .collect::<Vec<_>>()
        };

        let expected: Vec<_> = input().into_iter().flatten().collect();

        let par = input().into_par().num_threads(nt).chunk_size(chunk);
        let output: Vec<_> = par.flatten().collect();

        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn flatten_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || (0..n).collect::<Vec<_>>();
        let map = |i: usize| vec![i.to_string(), (i + 1).to_string()];

        let expected: Vec<_> = input().into_iter().map(&map).flatten().collect();

        let par = input().into_par().num_threads(nt).chunk_size(chunk);
        let output: Vec<_> = par.map(&map).flatten().collect();

        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn flatten_xap_filter_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || (0..n).map(|i| i.to_string()).collect::<Vec<_>>();
        let filter_map = |x: String| {
            x.starts_with('3')
                .then(|| vec![x.clone(), format!("{}!", x)])
        };

        let expected: Vec<_> = input()
            .clone()
            .into_iter()
            .filter_map(&filter_map)
            .flatten()
            .collect();

        let par = input().into_par().num_threads(nt).chunk_size(chunk);
        let output: Vec<_> = par.filter_map(filter_map).flatten().collect();

        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn flatten_xap_filter_xap(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || (0..n).map(|i| i.to_string()).collect::<Vec<_>>();
        let filter_map = |x: String| {
            x.starts_with('3')
                .then(|| vec![x.clone(), format!("{}!", x)])
        };
        let filter = |x: &Vec<String>| x.len() == 2;
        let map = |mut x: Vec<String>| {
            x.push("abc".to_string());
            x
        };
        let expected: Vec<_> = input()
            .clone()
            .into_iter()
            .filter_map(&filter_map)
            .filter(&filter)
            .map(&map)
            .flatten()
            .collect();

        let par = input().into_par().num_threads(nt).chunk_size(chunk);
        let output: Vec<_> = par
            .filter_map(filter_map)
            .filter(filter)
            .map(map)
            .flatten()
            .collect();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}
