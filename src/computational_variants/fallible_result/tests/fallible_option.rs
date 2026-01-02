use crate::{test_utils::*, *};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use test_case::test_matrix;

fn input<O: FromIterator<String>>(n: usize) -> O {
    let elem = |x: usize| (x + 10).to_string();
    (0..n).map(elem).collect()
}

#[test_matrix(NT, CHUNK)]
fn fallible_option_collect_empty(nt: &[usize], chunk: &[usize]) {
    let test = |_, nt, chunk| {
        let input = || input::<Vec<_>>(0);

        let par = input()
            .into_iter()
            .map(|x| Some(x))
            .collect::<Vec<_>>()
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .into_fallible_option();
        let output: Option<Vec<_>> = par.collect();

        assert_eq!(output, Some(Vec::new()));
    };
    test_n_nt_chunk(&[0], nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn fallible_option_collect_partial_success(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || input::<Vec<_>>(n);

        let par = input()
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .map(|x| (x != "50").then(|| x))
            .into_fallible_option()
            .filter(|x| !x.ends_with('9'))
            .flat_map(|x| [format!("{x}?"), x])
            .map(|x| format!("{x}!"));
        let output: Option<Vec<_>> = par.collect();

        assert_eq!(output, None);
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn fallible_option_collect_complete_success(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || input::<Vec<_>>(n);

        let expected: Vec<_> = input()
            .into_iter()
            .filter(|x| !x.ends_with('9'))
            .flat_map(|x| [format!("{x}?"), x])
            .map(|x| format!("{x}!"))
            .filter_map(|x| Some(x))
            .collect();

        let par = input()
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .map(|x| (x != "xyz").then(|| x))
            .into_fallible_option()
            .filter(|x| !x.ends_with('9'))
            .flat_map(|x| [format!("{x}?"), x])
            .map(|x| format!("{x}!"))
            .filter_map(|x| Some(x));
        let output: Option<Vec<_>> = par.collect();

        assert_eq!(output, Some(expected));
    };
    test_n_nt_chunk(n, nt, chunk, test);
}
