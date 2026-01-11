use super::utils::{make_u_filter, make_u_map};
use crate::{test_utils::*, *};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use orx_concurrent_vec::ConcurrentVec;
use test_case::test_matrix;

fn input<O: FromIterator<String>>(n: usize) -> O {
    let elem = |x: usize| (x + 10).to_string();
    (0..n).map(elem).collect()
}

fn sorted(mut x: Vec<String>) -> Vec<String> {
    x.sort();
    x
}

#[test_matrix(N, NT, CHUNK)]
fn for_each_empty(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || input::<Vec<_>>(n);

        let par = input()
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());
        let vec = ConcurrentVec::new();
        par.for_each(make_u_map(|x| {
            _ = vec.push(x);
        }));

        assert_eq!(sorted(vec.to_vec()), sorted(input()));
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn for_each_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || input::<Vec<_>>(n);
        let map = |x| format!("{}!", x);

        let par = input()
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());
        let par = par.map(make_u_map(map));
        let vec = ConcurrentVec::new();
        par.for_each(make_u_map(|x| {
            _ = vec.push(x);
        }));

        assert_eq!(
            sorted(vec.to_vec()),
            sorted(input().into_iter().map(map).collect())
        );
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn for_each_xap_flat_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || input::<Vec<_>>(n);
        let flat_map = |x: String| x.chars().map(|x| x.to_string()).collect::<Vec<_>>();

        let par = input()
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());
        let par = par.flat_map(make_u_map(flat_map));
        let vec = ConcurrentVec::new();
        par.for_each(make_u_map(|x| {
            _ = vec.push(x);
        }));

        assert_eq!(
            sorted(vec.to_vec()),
            sorted(input().into_iter().flat_map(flat_map).collect())
        );
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn for_each_xap_filter_map(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || input::<Vec<_>>(n);
        let filter_map = |x: String| x.starts_with('3').then_some(x);

        let par = input()
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());
        let par = par.filter_map(make_u_map(filter_map));
        let vec = ConcurrentVec::new();
        par.for_each(make_u_map(|x| {
            _ = vec.push(x);
        }));

        assert_eq!(
            sorted(vec.to_vec()),
            sorted(input().into_iter().filter_map(filter_map).collect())
        );
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn for_each_xap_filter_xap(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let input = || input::<Vec<_>>(n);
        let filter_map = |x: String| x.starts_with('3').then_some(x);
        let filter = |x: &String| x.ends_with('3');
        let flat_map = |x: String| x.chars().map(|x| x.to_string()).collect::<Vec<_>>();

        let par = input()
            .into_par()
            .num_threads(nt)
            .chunk_size(chunk)
            .using_clone("XyZw".to_string());
        let par = par
            .filter_map(make_u_map(filter_map))
            .filter(make_u_filter(&filter))
            .flat_map(make_u_map(flat_map));
        let vec = ConcurrentVec::new();
        par.for_each(make_u_map(|x| {
            _ = vec.push(x);
        }));

        assert_eq!(
            sorted(vec.to_vec()),
            sorted(
                input()
                    .into_iter()
                    .filter_map(filter_map)
                    .filter(filter)
                    .flat_map(flat_map)
                    .collect()
            )
        );
    };
    test_n_nt_chunk(n, nt, chunk, test);
}
