use crate::{test_utils::*, *};
use test_case::test_matrix;

fn input<O: FromIterator<String>>(n: usize) -> O {
    let elem = |x: usize| (x + 10).to_string();
    (0..n).map(elem).collect()
}

#[test_matrix(N, NT, CHUNK)]
fn count_empty(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let expected = n;
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output = par.count();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn count_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let expected = n;
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let par = par.map(|x| format!("{}!", x));
        let output = par.count();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn count_xap_flat_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let flat_map = |x: String| x.chars().map(|x| x.to_string()).collect::<Vec<_>>();
        let expected = input.clone().into_iter().flat_map(&flat_map).count();
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let par = par.flat_map(flat_map);
        let output = par.count();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn count_xap_filter_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let filter_map = |x: String| x.starts_with('3').then_some(x);
        let expected = input.clone().into_iter().filter_map(&filter_map).count();
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let par = par.filter_map(filter_map);
        let output = par.count();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn count_xap_filter_xap(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = input::<Vec<_>>(n);
        let filter_map = |x: String| x.starts_with('3').then_some(x);
        let filter = |x: &String| x.ends_with('3');
        let flat_map = |x: String| x.chars().map(|x| x.to_string()).collect::<Vec<_>>();
        let expected = input
            .clone()
            .into_iter()
            .filter_map(&filter_map)
            .filter(&filter)
            .flat_map(&flat_map)
            .count();
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let par = par.filter_map(filter_map).filter(filter).flat_map(flat_map);
        let output = par.count();
        assert_eq!(output, expected);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}
