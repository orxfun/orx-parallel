mod reduce_i64;
mod reduce_string;
mod utils;

use crate::utils::{test_different_params, test_reduce};
use orx_concurrent_iter::IterIntoConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_parallel::*;
use orx_split_vec::*;

#[test]
fn par_fmap_filter_par() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par: Par<_> = vec.into_par();

    assert_eq!(par.params(), Params::default());

    let par = par
        .flat_map(|x| vec![x; x])
        .filter(|x| x > &3)
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
fn par_fmap_filter_auto_when_zero_num_threads() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec
        .into_par()
        .flat_map(|x| vec![x; x])
        .filter(|x| x > &3)
        .num_threads(0);
    assert_eq!(par.params().num_threads, NumThreads::Auto);
}

#[test]
fn par_fmap_filter_auto_when_zero_chunk_size() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec
        .into_par()
        .flat_map(|x| vec![x; x])
        .filter(|x| x > &3)
        .chunk_size(0);
    assert_eq!(par.params().chunk_size, ChunkSize::Auto);
}

#[test]
fn par_fmap_filter_map() {
    let iter = [1, 42, 2, 111, 5, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.into_con_iter().into_par();
    let fmap = par
        .flat_map(|x| x.chars().collect::<Vec<_>>())
        .filter(|x| ['1', '9'].contains(x))
        .num_threads(2);
    let map = fmap.map(&String::from);
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &['1', '1', '1', '9'].map(String::from));
}

#[test]
fn par_fmap_filter_fmap() {
    let iter = [999, 3, 1, 0, 2].into_iter().skip(1);

    let par = iter.into_con_iter().into_par();
    let fmap = par.flat_map(|x| vec![x; x]).num_threads(2);
    let fmap = fmap.flat_map(|x| vec![x.to_string(), (x + 1).to_string()]);
    let filter = fmap.filter(|x| x.parse::<usize>().unwrap() > 2);
    let result = filter.collect_vec();

    assert_eq!(
        result.as_slice(),
        &[3, 4, 3, 4, 3, 4, 3, 3].map(|x| x.to_string())
    );
}

#[test]
fn par_fmap_filter_fmap_option() {
    let iter = [999, 3, 1, 0, 2].into_iter().skip(1);

    let par = iter.into_con_iter().into_par();
    let fmap = par.flat_map(|x| vec![x, x + 1, x + 2]).num_threads(4);
    let fmap = fmap.flat_map(|x| if x % 2 == 1 { Some(x) } else { None });
    let result = fmap.collect_vec();

    assert_eq!(result.as_slice(), &[3, 5, 1, 3, 1, 3]);
}

#[test]
fn par_fmap_filter_filter() {
    let iter = [999, 3, 1, 0, 2].into_iter().skip(1);

    let par = iter.into_con_iter().into_par();
    let fmap = par.flat_map(|x| vec![x, x + 1, x + 2]).num_threads(4);
    let fmap = fmap.flat_map(|x| if x % 2 == 1 { Some(x) } else { None });
    let filter = fmap.filter(|x| x > &2);
    let result = filter.collect_vec();

    assert_eq!(result.as_slice(), &[3, 5, 3, 3]);
}

#[test]
fn par_fmap_filter_collect() {
    fn test(num_threads: usize, chunk_size: usize) {
        let range = || 54..5648;

        let expected: Vec<_> = range()
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .collect();

        let result = range()
            .par()
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .collect();

        assert_eq!(result, expected);
    }
    test_different_params(test);
}

#[test]
fn par_fmap_filter_collect_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let range = || 54..5648;

        let expected: Vec<_> = range()
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .collect();

        let iter = range()
            .collect::<Vec<_>>()
            .into_iter()
            .take(10000)
            .into_con_iter()
            .into_par();
        let result = iter
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .collect_vec();

        assert_eq!(result, expected);
    }
    test_different_params(test)
}

#[test]
fn par_fmap_filter_collect_into() {
    fn test(num_threads: usize, chunk_size: usize) {
        let range1 = || 54..5648;
        let range2 = || 5648..6000;

        let expected: Vec<_> = range1()
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .collect();

        let result = range1()
            .into_con_iter()
            .into_par()
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .collect_into(SplitVec::new());

        assert_eq!(&result, &expected);

        let expected: Vec<_> = range1()
            .chain(range2())
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .collect();

        let result = range2()
            .collect::<Vec<_>>()
            .into_par()
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .collect_into(result);

        assert_eq!(&result, &expected);
    }
    test_different_params(test)
}

#[test]
fn par_fmap_filter_collect_into_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let range1 = || 54..5648;
        let range2 = || 5648..6000;

        let expected: Vec<_> = range1()
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .collect();

        let result = range1()
            .into_con_iter()
            .into_par()
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .collect_into(vec![]);

        assert_eq!(&result, &expected);

        let expected: Vec<_> = range1()
            .chain(range2())
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .collect();

        let result = range2()
            .collect::<Vec<_>>()
            .into_par()
            .flat_map(|x| [x * 2, x * 2 + 1])
            .filter(|x| x % 2 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .collect_into(result);

        assert_eq!(&result, &expected);
    }
    test_different_params(test)
}

#[test]
fn par_fmap_filter_collect_into_fixed_capacity_does_not_panic() {
    // TODO! this might be due to the todo in `merge_bag_and_pos_len` method. Revise afterwards.
    let vec = (54..5648).collect::<Vec<_>>();
    let iter = vec.into_iter().take(10000).into_con_iter().into_par();
    let fmap = iter
        .flat_map(|x| [x * 2])
        .filter(|x| x % 2 == 0)
        .num_threads(14)
        .chunk_size(64);
    let _ = fmap.collect_into(FixedVec::new(10));
}

#[test]
fn par_fmap_filter_collect_into_split_capacity_does_not_panic() {
    // TODO! this might be due to the todo in `merge_bag_and_pos_len` method. Revise afterwards.
    let vec = (54..5648).collect::<Vec<_>>();
    let iter = vec.into_iter().take(10000).into_con_iter().into_par();
    let map = iter
        .flat_map(|x| [x * 2])
        .filter(|x| x % 2 == 0)
        .num_threads(2)
        .chunk_size(64);
    let _ = map.collect_into(SplitVec::new());
}

// count

#[test]
fn par_fmap_filter_count() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.flat_map(|x| [x * 3, x * 2, x]).filter(|x| x % 2 == 0);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        let expected = (13..4785)
            .flat_map(|x| [x * 3, x * 2, x])
            .filter(|x| x % 2 == 0)
            .count();
        assert_eq!(par.count(), expected);
    }
    test_different_params(test)
}

// find

#[test]
fn par_fmap_filter_next() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.flat_map(|x| [x * 3, x * 2, x]).filter(|x| x % 2 == 0);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        let expected = (13..4785)
            .flat_map(|x| [x * 3, x * 2, x])
            .filter(|x| x % 2 == 0)
            .next();
        assert_eq!(par.first(), expected);
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
fn par_fmap_filter_find() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.flat_map(|x| [x * 3, x * 2, x]).filter(|x| x % 2 == 0);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        let expected = (13..4785)
            .flat_map(|x| [x * 3, x * 2, x])
            .filter(|x| x % 2 == 0)
            .find(|x| x >= &571);

        assert_eq!(par.find(|x| x >= &571), expected);
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
fn par_fmap_filter_reduce_i64() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (-100..len as i64)
                .map(|x| x as i64)
                .map(|x| x * 2 + 1)
                .collect::<Vec<_>>()
                .into_par()
                .num_threads(num_threads)
                .chunk_size(chunk_size)
                .map(|x| x - 1)
                .num_threads(num_threads)
                .flat_map(|x| vec![x / 2])
                .filter(|x| x >= &0)
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
fn par_fmap_filter_reduce_string() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (-100..len as i64)
                .map(|x| x as i64)
                .collect::<Vec<_>>()
                .into_par()
                .num_threads(num_threads)
                .flat_map(|x| [x.to_string()])
                .filter(|x| !x.starts_with('-'))
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
