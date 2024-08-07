mod reduce_i64;
mod reduce_string;
mod utils;

use crate::utils::*;
use orx_parallel::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn par_map_fil_par() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec.into_par();

    assert_eq!(par.params(), Params::default());

    let par = par
        .map(|x| x + 1)
        .filter(|x| x < &3)
        .chunk_size(64)
        .num_threads(8);
    assert_eq!(
        par.params(),
        Params {
            chunk_size: 64.into(),
            num_threads: 8.into()
        }
    );
}

#[test]
fn par_map_fil_auto_when_zero_num_threads() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec
        .into_par()
        .map(|x| x.to_string())
        .filter(|x| x.len() < 2)
        .num_threads(0);
    assert_eq!(par.params().num_threads, NumThreads::Auto);
}

#[test]
fn par_map_fil_auto_when_zero_chunk_size() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec
        .into_par()
        .map(|x| x.to_string())
        .filter(|x| x.is_empty())
        .chunk_size(0);
    assert_eq!(par.params().chunk_size, ChunkSize::Auto);
}

#[test]
fn par_map_fil_map() {
    let iter = [1, 42, 2, 111, 5, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.par();
    let map = par.map(|x| x.len()).num_threads(2);
    let filter = map.filter(|x| *x >= 2);
    let map = filter.map(|x| x * 2).chunk_size(2);
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &[4, 6, 8]);
}

#[test]
fn par_map_fil_fmap() {
    let iter = [1i64, 42, 2, 111, 5, 9876543210]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.par();
    let map = par.map(|x| x.len()).num_threads(2);
    let filter = map.filter(|x| *x >= 2);
    let map = filter.flat_map(|x| x.to_string().chars().collect::<Vec<_>>());
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &['2', '3', '1', '0']);
}

#[test]
fn par_map_fil_fmap_option() {
    let iter = [1i64, 42, 2, 111, 5, 9876543210]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.par();
    let map = par.map(|x| x.len()).num_threads(2);
    let filter = map.filter(|x| *x >= 2);
    let map = filter.flat_map(|x| if x % 2 == 0 { Some(x) } else { None });
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &[2, 10]);
}

#[test]
fn par_map_fil_filter() {
    let iter = [11, 42, 2, 111, 5, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.par();
    let map = par.map(|x| x.len()).num_threads(2);
    let filter = map.filter(|x| *x >= 2);
    let filter = filter.filter(|&x| x < 3);
    let result = filter.collect_vec();

    assert_eq!(result.as_slice(), [2]);
}

#[test]
fn par_map_fil_filtermap() {
    let iter = [11, 42, 2, 111, 5, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.par();
    let map = par.map(|x| x.len()).num_threads(2);
    let filter = map.filter(|x| *x >= 2);
    let filter = filter.filter_map(|x| some_if(x, |&x| x < 3));
    let result = filter.collect_vec();

    assert_eq!(result.as_slice(), [2]);
}

// collect

#[test]
fn par_map_fil_collect() {
    fn test(num_threads: usize, chunk_size: usize) {
        let vec = (54..5448).collect::<Vec<_>>();
        let iter = vec.iter().cloned().take(10000).par();
        let par = iter
            .map(|x| x * 2)
            .filter(|x| x % 3 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result = par.collect();

        let expected: Vec<_> = vec
            .into_iter()
            .map(|x| x * 2)
            .filter(|x| x % 3 == 0)
            .collect();

        assert_eq!(result.len(), expected.len());
        assert_eq!(result, expected);
    }
    test_different_params(test)
}

#[test]
fn par_map_fil_collect_x() {
    fn test(num_threads: usize, chunk_size: usize) {
        let vec = (54..5448).collect::<Vec<_>>();
        let iter = vec.iter().cloned().take(10000).par();
        let par = iter
            .map(|x| x * 2)
            .filter(|x| x % 3 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let mut result = par.collect_x().to_vec();
        result.sort();

        let expected: Vec<_> = vec
            .into_iter()
            .map(|x| x * 2)
            .filter(|x| x % 3 == 0)
            .collect();

        assert_eq!(result.len(), expected.len());
        assert_eq!(result, expected);
    }
    test_different_params(test)
}

#[test]
fn par_map_fil_collect_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let vec = (54..5448).collect::<Vec<_>>();
        let iter = vec.iter().cloned().take(10000).par();
        let par = iter
            .map(|x| x * 2)
            .filter(|x| x % 3 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result = par.collect_vec();

        let expected: Vec<_> = vec
            .into_iter()
            .map(|x| x * 2)
            .filter(|x| x % 3 == 0)
            .collect();

        assert_eq!(result.len(), expected.len());
        assert_eq!(result, expected);
    }
    test_different_params(test)
}

#[test]
fn par_map_fil_collect_into() {
    fn test(num_threads: usize, chunk_size: usize) {
        let n0 = 54;
        let n1 = 5448;
        let n2 = 6000;

        let par = (n0..n1)
            .collect::<Vec<_>>()
            .into_iter()
            .take(10000)
            .par()
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .map(|x| x * 2)
            .filter(|x| x % 3 == 0);
        let result1 = par.collect_into(SplitVec::new());

        let par = (n1..n2)
            .par()
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .map(|x| x * 2)
            .filter(|x| x % 3 == 0);
        let result2 = par.collect_into(result1);

        let expected: Vec<_> = (n0..n2).map(|x| x * 2).filter(|x| x % 3 == 0).collect();

        assert_eq!(result2.len(), expected.len());
        assert_eq!(result2, expected);
    }
    test_different_params(test)
}

#[test]
fn par_map_fil_collect_into_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let n0 = 54;
        let n1 = 5448;
        let n2 = 6000;

        let par = (n0..n1)
            .collect::<Vec<_>>()
            .into_iter()
            .take(10000)
            .par()
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .map(|x| x * 2)
            .filter(|x| x % 3 == 0);
        let result1 = par.collect_into(vec![]);

        let par = (n1..n2)
            .par()
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .map(|x| x * 2)
            .filter(|x| x % 3 == 0);
        let result2 = par.collect_into(result1);

        let expected: Vec<_> = (n0..n2).map(|x| x * 2).filter(|x| x % 3 == 0).collect();

        assert_eq!(result2.len(), expected.len());
        assert_eq!(result2, expected);
    }
    test_different_params(test)
}

// count

#[test]
fn par_map_fil_count() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).par();
        let par = par.map(|x| x + 7);
        let par = par.filter(|x| x % 3 == 2);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.count(), (13..4785).filter(|x| x % 3 == 2).count());
    }
    test_different_params(test)
}

#[test]
fn par_map_fil_foreach() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).par();
        let par = par.map(|x| x + 7).filter(|x| x % 3 == 2);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        let count = AtomicUsize::new(0);
        par.for_each(|x| {
            count.fetch_add(x, Ordering::AcqRel);
        });
        assert_eq!(
            count.load(Ordering::Relaxed),
            (13..4785).map(|x| x + 7).filter(|x| x % 3 == 2).sum()
        );
    }
    test_different_params(test)
}

#[test]
fn par_map_fil_all_any() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = || {
            (13..4785)
                .par()
                .map(|x| x + 7)
                .filter(|x| x % 3 == 2)
                .num_threads(num_threads)
                .chunk_size(chunk_size)
        };

        assert!(par().all(|x| x % 3 == 2));
        assert!(!par().all(|x| *x <= 4783));
        assert!(par().any(|x| *x > 3333));
        assert!(!par().any(|x| x % 3 == 1));
    }
    test_different_params(test)
}

// find

#[test]
fn par_map_fil_next() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).par();
        let par = par.map(|x| x + 7);
        let par = par.filter(|x| x % 3487 == 0);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.first_with_index(), Some((3487 - 7 - 13, 3487)));
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (0..0).par();
        let par = par.map(|x| x + 7);
        let par = par.filter(|x| x % 3487 == 0);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.first_with_index(), None);
    }
    test_different_params(test_empty);
}

#[test]
fn par_map_fil_find() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).par();
        let par = par.map(|x| x + 7);
        let par = par.filter(|x| x % 1000 == 0);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(
            par.find_with_index(|x| x > &1574),
            Some((2000 - 7 - 13, 2000))
        );
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).par();
        let par = par.map(|x| x + 7);
        let par = par.filter(|x| x % 1000 == 0);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.find_with_index(|x| x < &847), None);
    }
    test_different_params(test_empty);
}

// reduce

#[test]
fn par_map_fil_reduce_i64() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (-(len as i64)..(2 * len as i64))
                .map(|x| x * 2 + 1)
                .collect::<Vec<_>>()
                .into_par()
                .num_threads(num_threads)
                .chunk_size(chunk_size)
                .map(|x| x - 1)
                .num_threads(num_threads)
                .map(|x| x / 2)
                .filter(|x| *x >= 0 && *x < len as i64)
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
fn par_map_fil_reduce_string() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (-(len as i64)..(2 * len as i64))
                .map(|x| x * 2 + 1)
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .into_par()
                .num_threads(num_threads)
                .chunk_size(chunk_size)
                .map(|x| (x.parse::<i64>().expect("is-ok") - 1).to_string())
                .num_threads(num_threads)
                .map(|x| (x.parse::<i64>().expect("is-ok") / 2).to_string())
                .filter(|x| {
                    let x: i64 = x.parse().expect("is-ok");
                    x >= 0 && x < len as i64
                })
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
