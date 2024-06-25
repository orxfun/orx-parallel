mod reduce_i64;
mod reduce_string;
mod utils;

use crate::utils::{test_different_params, test_reduce};
use orx_concurrent_iter::IterIntoConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_parallel::*;
use orx_split_vec::*;

#[test]
fn par_fmap_par() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par: Par<_> = vec.into_par();

    assert_eq!(par.params(), Params::default());

    let par = par.flat_map(|x| vec![x; x]).chunk_size(64).num_threads(8);
    assert_eq!(
        par.params(),
        Params {
            chunk_size: 64.into(),
            num_threads: 8.into()
        }
    );
}

#[test]
fn par_fmap_auto_when_zero_num_threads() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec.into_par().flat_map(|x| vec![x; x]).num_threads(0);
    assert_eq!(par.params().num_threads, NumThreads::Auto);
}

#[test]
fn par_fmap_auto_when_zero_chunk_size() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec.into_par().flat_map(|x| vec![x; x]).chunk_size(0);
    assert_eq!(par.params().chunk_size, ChunkSize::Auto);
}

#[test]
fn par_fmap_map() {
    let iter = [1, 42, 2, 111, 5, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.into_con_iter().into_par();
    let fmap = par
        .flat_map(|x| x.chars().collect::<Vec<_>>())
        .num_threads(2);
    let map = fmap.map(&String::from);
    let result = map.collect_vec();

    assert_eq!(
        result.as_slice(),
        &['4', '2', '2', '1', '1', '1', '5', '9', '8', '7', '6'].map(String::from)
    );
}

#[test]
fn par_fmap_fmap() {
    let iter = [999, 3, 1, 0, 2].into_iter().skip(1);

    let par = iter.into_con_iter().into_par();
    let fmap = par.flat_map(|x| vec![x; x]).num_threads(2);
    let fmap = fmap.flat_map(|x| vec![x.to_string(), (x + 1).to_string()]);
    let result = fmap.collect_vec();

    assert_eq!(
        result.as_slice(),
        &[3, 4, 3, 4, 3, 4, 1, 2, 2, 3, 2, 3].map(|x| x.to_string())
    );
}

#[test]
fn par_fmap_fmap_option() {
    let iter = [999, 3, 1, 0, 2].into_iter().skip(1);

    let par = iter.into_con_iter().into_par();
    let fmap = par.flat_map(|x| vec![x; x]).num_threads(4);
    let fmap = fmap.flat_map(|x| if x % 2 == 1 { Some(x) } else { None });
    let result = fmap.collect_vec();

    assert_eq!(result.as_slice(), &[3, 3, 3, 1]);
}

#[test]
fn par_fmap_filter() {
    let iter = [999, 3, 1, 0, 2].into_iter().skip(1);

    let par = iter.into_con_iter().into_par();
    let fmap = par.flat_map(|x| vec![x; x]).num_threads(2);
    let filter = fmap.filter(|x| x % 2 == 1);
    let result = filter.collect_vec();

    assert_eq!(result.as_slice(), &[3, 3, 3, 1]);
}

// collect

#[test]
fn par_fmap_collect() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648).collect::<Vec<_>>().into_par();
        let fmap = iter
            .flat_map(|x| [x * 2, x * 2 + 1])
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result = fmap.collect();

        assert_eq!(result.len(), 2 * (5648 - 54));
        for i in 54..5648 {
            assert_eq!(result.get(2 * (i - 54)).cloned(), Some(i * 2));
            assert_eq!(result.get(2 * (i - 54) + 1).cloned(), Some(i * 2 + 1));
        }
    }
    test_different_params(test)
}

#[test]
fn par_fmap_collect_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648)
            .collect::<Vec<_>>()
            .into_iter()
            .take(10000)
            .into_con_iter()
            .into_par();
        let fmap = iter
            .flat_map(|x| [x * 2, x * 2 + 1])
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result = fmap.collect_vec();

        assert_eq!(result.len(), 2 * (5648 - 54));
        for i in 54..5648 {
            assert_eq!(result.get(2 * (i - 54)).cloned(), Some(i * 2));
            assert_eq!(result.get(2 * (i - 54) + 1).cloned(), Some(i * 2 + 1));
        }
    }
    test_different_params(test)
}

#[test]
fn par_fmap_collect_into() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648).collect::<Vec<_>>().into_par();
        let fmap = iter
            .flat_map(|x| [x * 2, x * 2 + 1])
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result1 = fmap.collect_into(SplitVec::new());

        assert_eq!(result1.len(), 2 * (5648 - 54));
        for i in 54..5648 {
            assert_eq!(result1.get(2 * (i - 54)).cloned(), Some(i * 2));
            assert_eq!(result1.get(2 * (i - 54) + 1).cloned(), Some(i * 2 + 1));
        }

        let iter = (5648..6000).collect::<Vec<_>>().into_par();
        let fmap = iter
            .flat_map(|x| [x * 2, x * 2 + 1])
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result2 = fmap.collect_into(result1);

        assert_eq!(result2.len(), 2 * (6000 - 54));
        for i in 54..6000 {
            assert_eq!(result2.get(2 * (i - 54)).cloned(), Some(i * 2));
            assert_eq!(result2.get(2 * (i - 54) + 1).cloned(), Some(i * 2 + 1));
        }
    }
    test_different_params(test)
}

#[test]
fn par_fmap_collect_into_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648).collect::<Vec<_>>().into_par();
        let fmap = iter
            .flat_map(|x| [x * 2, x * 2 + 1])
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result1 = fmap.collect_into(vec![]);

        assert_eq!(result1.len(), 2 * (5648 - 54));
        for i in 54..5648 {
            assert_eq!(result1.get(2 * (i - 54)).cloned(), Some(i * 2));
            assert_eq!(result1.get(2 * (i - 54) + 1).cloned(), Some(i * 2 + 1));
        }

        let iter = (5648..6000).collect::<Vec<_>>().into_par();
        let fmap = iter
            .flat_map(|x| [x * 2, x * 2 + 1])
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result2 = fmap.collect_into(result1);

        assert_eq!(result2.len(), 2 * (6000 - 54));
        for i in 54..6000 {
            assert_eq!(result2.get(2 * (i - 54)).cloned(), Some(i * 2));
            assert_eq!(result2.get(2 * (i - 54) + 1).cloned(), Some(i * 2 + 1));
        }
    }
    test_different_params(test)
}

#[test]
fn par_fmap_collect_into_fixed_capacity_does_not_panic() {
    // TODO! this might be due to the todo in `merge_bag_and_pos_len` method. Revise afterwards.
    let vec = (54..5648).collect::<Vec<_>>();
    let iter = vec.into_iter().take(10000).into_con_iter().into_par();
    let fmap = iter.flat_map(|x| [x * 2]).num_threads(14).chunk_size(64);
    let _ = fmap.collect_into(FixedVec::new(5648 - 54 - 1));
}

#[test]
fn par_fmap_collect_into_split_capacity_does_not_panic() {
    // TODO! this might be due to the todo in `merge_bag_and_pos_len` method. Revise afterwards.
    let vec = (54..5648).collect::<Vec<_>>();
    let iter = vec.into_iter().take(10000).into_con_iter().into_par();
    let map = iter.flat_map(|x| [x * 2]).num_threads(2).chunk_size(64);
    let _ = map.collect_into(SplitVec::new());
}

// count

#[test]
fn par_fmap_count() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.flat_map(|x| [x * 3, x * 2, x]);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.count(), 3 * (4785 - 13));
    }
    test_different_params(test)
}

// find

#[test]
fn par_fmap_next() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.flat_map(|x| [x * 3, x + 1]);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.first(), Some(13 * 3));
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (0..0).into_con_iter().into_par();
        let par = par.flat_map(|x| [x * 3]);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.first(), None);
    }
    test_different_params(test_empty);
}

#[test]
fn par_fmap_find() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.flat_map(|x| [x * 3, x + 1]);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);

        assert_eq!(par.find(|x| x >= &40), Some(42));
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.flat_map(|x| [x / 3, x + 3, x - 12]);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.find(|x| x % 13333 == 0), None);
    }
    test_different_params(test_empty);
}

// reduce

#[test]
fn par_fmap_reduce_i64() {
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
                .flat_map(|x| vec![x / 2])
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
fn par_fmap_reduce_string() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (0..len)
                .map(|x| x as i64)
                .collect::<Vec<_>>()
                .into_par()
                .num_threads(num_threads)
                .flat_map(|x| [x.to_string()])
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
