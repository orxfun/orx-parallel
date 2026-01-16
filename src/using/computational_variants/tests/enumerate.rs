use crate::{test_utils::*, *};
use alloc::string::ToString;
use alloc::vec::Vec;
use test_case::test_matrix;

#[test_matrix(N, NT, CHUNK)]
fn enumerate_sequential(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let par = (0..n)
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());
        let par = par.enumerate();
        let vec: Vec<_> = par.collect();

        vec.iter().for_each(|(idx, value)| {
            assert_eq!(idx, value);
        });
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn enumerate_nested(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let par = (0..n)
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());

        let par = par.enumerate().enumerate();
        let vec: Vec<_> = par.collect();

        vec.iter().for_each(|(idx_outer, (idx_inner, value))| {
            assert_eq!(idx_outer, value);
            assert_eq!(idx_inner, value);
        });
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn enumerate_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let par = (0..n)
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());

        let par = par
            .enumerate()
            .map(|_u, (idx, value)| (idx + 1, value.pow(2)));
        let vec: Vec<_> = par.collect();

        vec.iter().for_each(|(idx_outer, value)| {
            assert_eq!((idx_outer - 1).pow(2), *value);
        });
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn enumerate_xap_filter_xap(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let par = (0..n)
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());

        let par = par.enumerate().filter_map(|_u, (idx, value)| {
            if idx.is_multiple_of(3) {
                Some((idx + 1, value.pow(2)))
            } else {
                None
            }
        });
        let vec: Vec<_> = par.collect();

        vec.iter().for_each(|(idx_outer, value)| {
            assert_eq!((idx_outer - 1).pow(2), *value);
            assert!(value.isqrt().is_multiple_of(3));
        });
    };
    test_n_nt_chunk(n, nt, chunk, test);
}
