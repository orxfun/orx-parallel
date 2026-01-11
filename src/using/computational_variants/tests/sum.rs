use crate::using::computational_variants::tests::utils::{make_u_filter, make_u_map};
use crate::{test_utils::*, *};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use test_case::test_matrix;

fn input<O: FromIterator<String>>(n: usize) -> O {
    let elem = |x: usize| (x + 10).to_string();
    (0..n).map(elem).collect()
}

#[test_matrix(N, NT, CHUNK)]
fn sum_empty(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input: Vec<_> = input::<Vec<_>>(n).iter().map(|x| x.len()).collect();
        let expected: usize = input.iter().sum();
        let par = input
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());
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
        let par = input
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());
        let par = par.map(make_u_map(map));
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
        let par = input
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());
        let par = par.flat_map(make_u_map(flat_map));
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
        let par = input
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());
        let par = par.filter_map(make_u_map(filter_map));
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
        let expected: usize = input
            .clone()
            .into_iter()
            .filter_map(&filter_map)
            .filter(&filter)
            .flat_map(&flat_map)
            .sum();
        let par = input
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());
        let par = par
            .filter_map(make_u_map(filter_map))
            .filter(make_u_filter(&filter))
            .flat_map(make_u_map(flat_map));
        let output = par.sum();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}
