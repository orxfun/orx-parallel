mod reduce_i64;
mod reduce_string;
mod utils;

use crate::utils::*;
use orx_concurrent_iter::IterIntoConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_parallel::*;
use orx_split_vec::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn par_map_par() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par: ParEmpty<_> = vec.into_par();

    assert_eq!(par.params(), Params::default());

    let par = par.map(|x| x + 1).chunk_size(64).num_threads(8);
    assert_eq!(
        par.params(),
        Params {
            chunk_size: 64.into(),
            num_threads: 8.into()
        }
    );
}

#[test]
fn par_map_auto_when_zero_num_threads() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec.into_par().map(|x| x.to_string()).num_threads(0);
    assert_eq!(par.params().num_threads, NumThreads::Auto);
}

#[test]
fn par_map_auto_when_zero_chunk_size() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec.into_par().map(|x| x.to_string()).chunk_size(0);
    assert_eq!(par.params().chunk_size, ChunkSize::Auto);
}

#[test]
fn par_map_map() {
    let iter = [1, 42, 2, 111, 5, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.par();
    let map = par.map(|x| x.len()).num_threads(2);
    let map = map.map(|x| x * 2).chunk_size(2);
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &[4, 2, 6, 2, 8]);
}

#[test]
fn par_map_fmap() {
    let iter = [1i64, 42, 2, 111, 5, 9876543210]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.par();
    let map = par.map(|x| x.len()).num_threads(2);
    let map = map.flat_map(|x| x.to_string().chars().collect::<Vec<_>>());
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &['2', '1', '3', '1', '1', '0']);
}

#[test]
fn par_map_fmap_option() {
    let iter = [1i64, 42, 2, 111, 5, 9876543210]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.par();
    let map = par.map(|x| x.len()).num_threads(2);
    let map = map.flat_map(|x| if x % 2 == 0 { Some(x) } else { None });
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &[2, 10]);
}

#[test]
fn par_map_filter() {
    let iter = [11, 42, 2, 111, 5, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.par();
    let map = par.map(|x| x.len()).num_threads(2);
    let filter = map.filter(|x| x % 2 == 0).num_threads(2);
    let result = filter.collect_vec();

    assert_eq!(result.as_slice(), [2, 4]);
}

#[test]
fn par_map_filtermap() {
    let iter = [11, 42, 2, 111, 5, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.par();
    let map = par.map(|x| x.len()).num_threads(2);
    let filter = map.filter_map(|x| ok_if(x, |x| x % 2 == 0)).num_threads(2);
    let result = filter.collect_vec();

    assert_eq!(result.as_slice(), [2, 4]);
}

// collect

#[test]
fn par_map_collect() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648).collect::<Vec<_>>().into_par();
        let map = iter
            .map(|x| x * 2)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result = map.collect();

        assert_eq!(result.len(), 5648 - 54);
        for i in 54..5648 {
            assert_eq!(result.get(i - 54).cloned(), Some(i * 2));
        }
    }
    test_different_params(test)
}

#[test]
fn par_map_collect_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648)
            .collect::<Vec<_>>()
            .into_iter()
            .take(10000)
            .into_con_iter()
            .into_par();
        let map = iter
            .map(|x| x * 2)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result = map.collect_vec();

        assert_eq!(result.len(), 5648 - 54);
        for i in 54..5648 {
            assert_eq!(result.get(i - 54).cloned(), Some(i * 2));
        }
    }
    test_different_params(test)
}

#[test]
fn par_map_collect_into() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648).collect::<Vec<_>>().into_par();
        let map = iter
            .map(|x| x * 2)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result1 = map.collect_into(SplitVec::new());

        assert_eq!(result1.len(), 5648 - 54);
        for i in 54..5648 {
            assert_eq!(result1.get(i - 54).cloned(), Some(i * 2));
        }

        let iter = (5648..6000).collect::<Vec<_>>().into_par();
        let map = iter
            .map(|x| x * 2)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result2 = map.collect_into(result1);

        assert_eq!(result2.len(), 6000 - 54);
        for i in 54..6000 {
            assert_eq!(result2.get(i - 54).cloned(), Some(i * 2));
        }
    }
    test_different_params(test)
}

#[test]
fn par_map_collect_into_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648).collect::<Vec<_>>().into_par();
        let map = iter
            .map(|x| x * 2)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result1 = map.collect_into(vec![]);

        assert_eq!(result1.len(), 5648 - 54);
        for i in 54..5648 {
            assert_eq!(result1.get(i - 54).cloned(), Some(i * 2));
        }

        let iter = (5648..6000).collect::<Vec<_>>().into_par();
        let map = iter
            .map(|x| x * 2)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result2 = map.collect_into(result1);

        assert_eq!(result2.len(), 6000 - 54);
        for i in 54..6000 {
            assert_eq!(result2.get(i - 54).cloned(), Some(i * 2));
        }
    }
    test_different_params(test)
}

#[test]
#[should_panic]
fn par_map_collect_into_fixed_capacity_panic() {
    let vec = (54..5648).collect::<Vec<_>>();
    let iter = vec
        .into_iter()
        .take(10000)
        .filter(|x| x < &50000)
        .into_con_iter()
        .into_par();
    let map = iter.map(|x| x * 2).num_threads(2).chunk_size(64);
    let _ = map.collect_into(FixedVec::new(5648 - 54 - 1));
}

#[test]
#[should_panic]
fn par_map_collect_into_split_capacity_panic() {
    // TODO: there is no reason for Doubling and Recursive to panic
    let vec = (54..5648).collect::<Vec<_>>();
    let iter = vec
        .into_iter()
        .take(10000)
        .filter(|x| x < &87544)
        .into_con_iter()
        .into_par();
    let map = iter.map(|x| x * 2).num_threads(2).chunk_size(64);
    let _ = map.collect_into(SplitVec::new());
}

// count

#[test]
fn par_map_count() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).par();
        let par = par.map(|x| x * 3);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.count(), 4785 - 13);
    }
    test_different_params(test)
}

#[test]
fn par_map_foreach() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).par();
        let par = par.map(|x| x + 7);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        let count = AtomicUsize::new(0);
        par.for_each(|x| {
            count.fetch_add(x, Ordering::AcqRel);
        });
        assert_eq!(
            count.load(Ordering::Relaxed),
            (13..4785).map(|x| x + 7).sum()
        );
    }
    test_different_params(test)
}

// find

#[test]
fn par_map_next() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).par();
        let par = par.map(|x| x * 3);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.first_with_index(), Some((0, 13 * 3)));
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (0..0).par();
        let par = par.map(|x| x * 3);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.first_with_index(), None);
    }
    test_different_params(test_empty);
}

#[test]
fn par_map_find() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).par();
        let par = par.map(|x| x * 3);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.find_with_index(|x| x >= &40), Some((14 - 13, 42)));
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).par();
        let par = par.map(|x| x / 3);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.find_with_index(|x| x % 13333 == 0), None);
    }
    test_different_params(test_empty);
}

// reduce

#[test]
fn par_map_reduce_i64() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (0..len)
                .map(|x| x as i64)
                .map(|x| x * 2 + 1)
                .collect::<Vec<_>>()
                .into_par()
                .num_threads(num_threads)
                .chunk_size(chunk_size)
                .map(|x| x - 1)
                .num_threads(num_threads)
                .map(|x| x / 2)
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
fn par_map_reduce_string() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (0..len)
                .map(|x| x as i64)
                .collect::<Vec<_>>()
                .into_par()
                .num_threads(num_threads)
                .map(|x| x.to_string())
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
