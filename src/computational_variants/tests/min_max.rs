use crate::{test_utils::*, *};
use std::cmp::Ordering;
use test_case::test_matrix;

fn input<O: FromIterator<String>>(n: usize) -> O {
    let elem = |x: usize| (x + 10).to_string();
    (0..n).map(elem).collect()
}

fn cmp(a: &usize, b: &usize) -> Ordering {
    match a < b {
        true => Ordering::Less,
        false => Ordering::Greater,
    }
}

fn key(a: &usize) -> u64 {
    *a as u64 + 10
}

#[test_matrix(N, NT, CHUNK)]
fn min_max_empty(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || {
            input::<Vec<_>>(n)
                .iter()
                .map(|x| x.parse::<usize>().unwrap())
                .collect::<Vec<_>>()
        };

        let par = || input().into_par().num_threads(nt).chunk_size(chunk);

        assert_eq!(par().min(), input().iter().min().copied());
        assert_eq!(par().max(), input().iter().max().copied());

        assert_eq!(par().min_by(cmp), input().into_iter().min_by(cmp));
        assert_eq!(par().max_by(cmp), input().into_iter().max_by(cmp));

        assert_eq!(par().min_by_key(key), input().into_iter().min_by_key(key));
        assert_eq!(par().max_by_key(key), input().into_iter().max_by_key(key));
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn min_max_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || input::<Vec<_>>(n);
        let map = |x: String| x.parse::<usize>().unwrap();

        let par = || {
            input()
                .into_par()
                .num_threads(nt)
                .chunk_size(chunk)
                .map(&map)
        };

        assert_eq!(par().min(), input().into_iter().map(&map).min());
        assert_eq!(par().max(), input().into_iter().map(&map).max());

        assert_eq!(par().min_by(cmp), input().into_iter().map(&map).min_by(cmp));
        assert_eq!(par().max_by(cmp), input().into_iter().map(&map).max_by(cmp));

        assert_eq!(
            par().min_by_key(key),
            input().into_iter().map(&map).min_by_key(key)
        );
        assert_eq!(
            par().max_by_key(key),
            input().into_iter().map(&map).max_by_key(key)
        );
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn min_max_xap_flat_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || input::<Vec<_>>(n);
        let flat_map = |x: String| {
            let n = x.len();
            let a = x.parse::<usize>().unwrap();
            (0..n).map(|i| a + i).collect::<Vec<_>>()
        };

        let par = || {
            input()
                .into_par()
                .num_threads(nt)
                .chunk_size(chunk)
                .flat_map(&flat_map)
        };

        assert_eq!(par().min(), input().into_iter().flat_map(&flat_map).min());
        assert_eq!(par().max(), input().into_iter().flat_map(&flat_map).max());

        assert_eq!(
            par().min_by(cmp),
            input().into_iter().flat_map(&flat_map).min_by(cmp)
        );
        assert_eq!(
            par().max_by(cmp),
            input().into_iter().flat_map(&flat_map).max_by(cmp)
        );

        assert_eq!(
            par().min_by_key(key),
            input().into_iter().flat_map(&flat_map).min_by_key(key)
        );
        assert_eq!(
            par().max_by_key(key),
            input().into_iter().flat_map(&flat_map).max_by_key(key)
        );
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn min_max_xap_filter_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || input::<Vec<_>>(n);
        let filter_map = |x: String| x.starts_with('3').then(|| x.parse::<usize>().unwrap());

        let par = || {
            input()
                .into_par()
                .num_threads(nt)
                .chunk_size(chunk)
                .filter_map(&filter_map)
        };

        assert_eq!(
            par().min(),
            input().into_iter().filter_map(&filter_map).min()
        );
        assert_eq!(
            par().max(),
            input().into_iter().filter_map(&filter_map).max()
        );

        assert_eq!(
            par().min_by(cmp),
            input().into_iter().filter_map(&filter_map).min_by(cmp)
        );
        assert_eq!(
            par().max_by(cmp),
            input().into_iter().filter_map(&filter_map).max_by(cmp)
        );

        assert_eq!(
            par().min_by_key(key),
            input().into_iter().filter_map(&filter_map).min_by_key(key)
        );
        assert_eq!(
            par().max_by_key(key),
            input().into_iter().filter_map(&filter_map).max_by_key(key)
        );
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn min_max_xap_filter_xap(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || input::<Vec<_>>(n);
        let filter_map = |x: String| x.starts_with('3').then_some(x);
        let filter = |x: &String| x.ends_with('3');
        let flat_map = |x: String| {
            let n = x.len();
            let a = x.parse::<usize>().unwrap();
            (0..n).map(|i| a + i).collect::<Vec<_>>()
        };

        let par = || {
            input()
                .into_par()
                .num_threads(nt)
                .chunk_size(chunk)
                .filter_map(&filter_map)
                .filter(&filter)
                .flat_map(&flat_map)
        };

        assert_eq!(
            par().min(),
            input()
                .into_iter()
                .filter_map(&filter_map)
                .filter(&filter)
                .flat_map(&flat_map)
                .min()
        );
        assert_eq!(
            par().max(),
            input()
                .into_iter()
                .filter_map(&filter_map)
                .filter(&filter)
                .flat_map(&flat_map)
                .max()
        );

        assert_eq!(
            par().min_by(cmp),
            input()
                .into_iter()
                .filter_map(&filter_map)
                .filter(&filter)
                .flat_map(&flat_map)
                .min_by(cmp)
        );
        assert_eq!(
            par().max_by(cmp),
            input()
                .into_iter()
                .filter_map(&filter_map)
                .filter(&filter)
                .flat_map(&flat_map)
                .max_by(cmp)
        );

        assert_eq!(
            par().min_by_key(key),
            input()
                .into_iter()
                .filter_map(&filter_map)
                .filter(&filter)
                .flat_map(&flat_map)
                .min_by_key(key)
        );
        assert_eq!(
            par().max_by_key(key),
            input()
                .into_iter()
                .filter_map(&filter_map)
                .filter(&filter)
                .flat_map(&flat_map)
                .max_by_key(key)
        );
    };
    test_n_nt_chunk(n, nt, chunk, test);
}
