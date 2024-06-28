mod reduce_i64;
mod reduce_string;
mod utils;

use crate::utils::*;
use orx_concurrent_iter::IterIntoConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_parallel::*;
use orx_split_vec::*;

#[test]
fn par_filtermap_par() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par: ParEmpty<_> = vec.into_par();

    assert_eq!(par.params(), Params::default());

    let par = par.filter_map(|x| Some(x)).chunk_size(64).num_threads(8);
    assert_eq!(
        par.params(),
        Params {
            chunk_size: 64.into(),
            num_threads: 8.into()
        }
    );
}

#[test]
fn par_filtermap_auto_when_zero_num_threads() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec.into_par().filter_map(|x| Some(x)).num_threads(0);
    assert_eq!(par.params().num_threads, NumThreads::Auto);
}

#[test]
fn par_filtermap_auto_when_zero_chunk_size() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec.into_par().filter_map(|x| Some(x)).chunk_size(0);
    assert_eq!(par.params().chunk_size, ChunkSize::Auto);
}

#[test]
fn par_filtermap_map() {
    let iter = [1, 42, 2, 111, 5, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.into_con_iter().into_par();
    let filtermap = par
        .filter_map(|x| some_if(x, |x| x.len() % 2 == 0))
        .num_threads(2);
    let map = filtermap.map(&String::from);
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &["42", "9876"].map(String::from));
}

#[test]
fn par_filtermap_flatmap() {
    let iter = [999, 3, 1, 0, 2].into_iter().skip(1);

    let par = iter.into_con_iter().into_par();
    let filtermap = par.filter_map(|x| ok_if(x, |x| x % 2 == 0)).num_threads(2);
    let filtermap = filtermap.flat_map(|x| vec![x.to_string(), (x + 1).to_string()]);
    let result = filtermap.collect_vec();

    assert_eq!(result.as_slice(), &[0, 1, 2, 3].map(|x| x.to_string()));
}

#[test]
fn par_filtermap_flatmap_option() {
    let iter = [999, 3, 1, 0, 2].into_iter().skip(1);

    let par = iter.into_con_iter().into_par();
    let filtermap = par.filter_map(|x| ok_if(x, |x| x % 2 == 0)).num_threads(4);
    let filtermap = filtermap.flat_map(|x| if x > 1 { Some(x) } else { None });
    let result = filtermap.collect_vec();

    assert_eq!(result.as_slice(), &[2]);
}

#[test]
fn par_filtermap_filter() {
    let iter = [999, 3, 1, 0, 2].into_iter().skip(1);

    let par = iter.into_con_iter().into_par();
    let filtermap = par.filter_map(|x| ok_if(x, |x| x % 2 == 0)).num_threads(2);
    let filter = filtermap.filter(|x| *x > 1);
    let result = filter.collect_vec();

    assert_eq!(result.as_slice(), &[2]);
}

#[test]
fn par_filtermap_filtermap() {
    let iter = [999, 3, 1, 0, 2].into_iter().skip(1);

    let par = iter.into_con_iter().into_par();
    let filtermap = par.filter_map(|x| ok_if(x, |x| x % 2 == 0)).num_threads(2);
    let filter = filtermap.filter_map(|x| some_if(x, |x| *x > 1));
    let result = filter.collect_vec();

    assert_eq!(result.as_slice(), &[2]);
}

// collect

#[test]
fn par_filtermap_collect() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648).collect::<Vec<_>>().into_par();
        let filtermap = iter
            .filter_map(|x| ok_if(x, |x| x % 2 == 0))
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result = filtermap.collect();

        assert_eq!(result.len(), (5648 - 54) / 2);

        for i in (54..5648).step_by(2) {
            assert_eq!(result.get((i - 54) / 2).cloned(), Some(i));
        }
    }
    test_different_params(test)
}

#[test]
fn par_filtermap_collect_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648)
            .collect::<Vec<_>>()
            .into_iter()
            .take(10000)
            .into_con_iter()
            .into_par();
        let filtermap = iter
            .filter_map(|x| some_if(x, |x| x % 2 == 0))
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result = filtermap.collect_vec();

        assert_eq!(result.len(), (5648 - 54) / 2);
        for i in (54..5648).step_by(2) {
            assert_eq!(result.get((i - 54) / 2).cloned(), Some(i));
        }
    }
    test_different_params(test)
}

#[test]
fn par_filtermap_collect_into() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648).collect::<Vec<_>>().into_par();
        let filtermap = iter
            .filter_map(|x| ok_if(x, |x| x % 2 == 0))
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result1 = filtermap.collect_into(SplitVec::new());

        assert_eq!(result1.len(), (5648 - 54) / 2);
        for i in (54..5648).step_by(2) {
            assert_eq!(result1.get((i - 54) / 2).cloned(), Some(i));
        }

        let iter = (5648..6000).collect::<Vec<_>>().into_par();
        let filtermap = iter
            .filter_map(|x| some_if(x, |x| x % 2 == 0))
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result2 = filtermap.collect_into(result1);

        assert_eq!(result2.len(), (6000 - 54) / 2);
        for i in (54..6000).step_by(2) {
            assert_eq!(result2.get((i - 54) / 2).cloned(), Some(i));
        }
    }
    test_different_params(test)
}

#[test]
fn par_filtermap_collect_into_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5648).collect::<Vec<_>>().into_par();
        let filtermap = iter
            .filter_map(|x| some_if(x, |x| x % 2 == 0))
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result1 = filtermap.collect_into(SplitVec::new());

        assert_eq!(result1.len(), (5648 - 54) / 2);
        for i in (54..5648).step_by(2) {
            assert_eq!(result1.get((i - 54) / 2).cloned(), Some(i));
        }

        let iter = (5648..6000).collect::<Vec<_>>().into_par();
        let filtermap = iter
            .filter_map(|x| some_if(x, |x| x % 2 == 0))
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result2 = filtermap.collect_into(result1);

        assert_eq!(result2.len(), (6000 - 54) / 2);
        for i in (54..6000).step_by(2) {
            assert_eq!(result2.get((i - 54) / 2).cloned(), Some(i));
        }
    }
    test_different_params(test)
}

#[test]
fn par_filtermap_collect_into_fixed_capacity_does_not_panic() {
    // TODO! this might be due to the todo in `merge_bag_and_pos_len` method. Revise afterwards.
    let vec = (54..5648).collect::<Vec<_>>();
    let iter = vec.into_iter().take(10000).into_con_iter().into_par();
    let filtermap = iter
        .filter_map(|x| some_if(x, |x| x % 2 == 0))
        .num_threads(14)
        .chunk_size(64);
    let _ = filtermap.collect_into(FixedVec::new((5648 - 54) / 2 - 1));
}

#[test]
fn par_filtermap_collect_into_split_capacity_does_not_panic() {
    // TODO! this might be due to the todo in `merge_bag_and_pos_len` method. Revise afterwards.
    let vec = (54..5648).collect::<Vec<_>>();
    let iter = vec.into_iter().take(10000).into_con_iter().into_par();
    let map = iter
        .filter_map(|x| some_if(x, |x| x % 2 == 0))
        .num_threads(2)
        .chunk_size(64);
    let _ = map.collect_into(SplitVec::new());
}

// count

#[test]
fn par_filtermap_count() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.filter_map(|x| some_if(x, |x| x % 2 == 1));
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.count(), (4785 - 13) / 2);
    }
    test_different_params(test)
}

// find

#[test]
fn par_filtermap_next() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.filter_map(|x| some_if(x, |x| x % 19 == 0));
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.first(), Some(19));
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (0..0).into_con_iter().into_par();
        let par = par.filter_map(|x| some_if(x, |x| x % 19 == 11));
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.first(), None);
    }
    test_different_params(test_empty);
}

#[test]
fn par_filtermap_find() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.filter_map(|x| some_if(x, |x| x % 3 == 0));
        let par = par.num_threads(num_threads).chunk_size(chunk_size);

        assert_eq!(par.find(|x| x >= &40), Some(42));
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.filter_map(|x| some_if(x, |x| x % 3 == 0));
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.find(|x| x % 13333 == 0), None);
    }
    test_different_params(test_empty);
}

// reduce

#[test]
fn par_filtermap_reduce_i64() {
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
                .filter_map(|x| Some(x / 2))
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
fn par_filtermap_reduce_string() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (0..len)
                .map(|x| x as i64)
                .collect::<Vec<_>>()
                .into_par()
                .num_threads(num_threads)
                .filter_map(|x| ok_if(x.to_string(), |_| true))
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
