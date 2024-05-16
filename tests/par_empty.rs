mod reduce_i64;
mod reduce_string;
mod utils;

use orx_concurrent_iter::IterIntoConcurrentIter;
use orx_parallel::*;
use std::num::NonZeroUsize;

use crate::utils::{test_different_params, test_reduce};

#[test]
fn par_empty_into_par() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par: Par<_> = vec.into_par();

    assert_eq!(par.params(), Params::default());

    let par = par.chunk_size(64).num_threads(8);
    assert_eq!(
        par.params(),
        Params {
            chunk_size: Some(NonZeroUsize::new(64).unwrap()),
            num_threads: Some(NonZeroUsize::new(8).unwrap())
        }
    );
}

#[test]
#[should_panic]
fn par_empty_panics_when_zero_num_threads() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let _ = vec.into_par().num_threads(0);
}

#[test]
#[should_panic]
fn par_empty_panics_when_zero_chunk_size() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let _ = vec.into_par().chunk_size(0);
}

#[test]
fn par_empty_map() {
    let iter = [1, 42, 2, 111, 5, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.into_con_iter().into_par();
    let map = par.map(|x| x.len()).num_threads(2);
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &[2, 1, 3, 1, 4]);
}

#[test]
fn par_empty_filter() {
    let iter = [11, 42, 2, 111, 5, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.into_con_iter().into_par();
    let filter = par.filter(|x| x.len() % 2 == 0).num_threads(2);
    let result = filter.collect_vec();

    assert_eq!(result.as_slice(), [42, 9876].map(|x| x.to_string()));
}

// count

#[test]
fn par_empty_count() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.count(), 4785 - 13);
    }
    test_different_params(test)
}

// find

#[test]
fn par_empty_next() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.next(), Some((0, 13)));
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (0..0).into_con_iter().into_par();
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.next(), None);
    }
    test_different_params(test_empty);
}

#[test]
fn par_empty_find() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.find(|x| x >= &489), Some((489 - 13, 489)));
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.find(|x| x % 13333 == 0), None);
    }
    test_different_params(test_empty);
}

// reduce

#[test]
fn par_empty_reduce_i64() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (0..len)
                .map(|x| x as i64)
                .collect::<Vec<_>>()
                .into_par()
                .num_threads(num_threads)
                .chunk_size(chunk_size)
        };

        reduce_i64::reduce(data(), len);
        reduce_i64::sum(data(), len);
        reduce_i64::min(data(), len);
        reduce_i64::max(data(), len);
        reduce_i64::min_by(data(), len);
        reduce_i64::max_by(data(), len);
        reduce_i64::min_by_key(data(), len);
        reduce_i64::max_by_key(data(), len);
    }
    test_reduce(test)
}

#[test]
fn par_empty_reduce_string() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (0..len)
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .into_par()
                .num_threads(num_threads)
                .chunk_size(chunk_size)
        };

        reduce_string::reduce(data(), len);
        reduce_string::min(data(), len);
        reduce_string::max(data(), len);
        reduce_string::min_by(data(), len);
        reduce_string::max_by(data(), len);
        reduce_string::min_by_key(data(), len);
        reduce_string::max_by_key(data(), len);
    }
    test_reduce(test)
}
