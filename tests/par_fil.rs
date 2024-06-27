mod reduce_i64;
mod reduce_string;
mod utils;

use crate::utils::*;
use orx_concurrent_iter::IterIntoConcurrentIter;
use orx_parallel::*;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;

#[test]
fn par_fil_par() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par: Par<_> = vec.into_par();

    assert_eq!(par.params(), Params::default());

    let par = par.filter(|x| x % 3 == 1).chunk_size(64).num_threads(8);
    assert_eq!(
        par.params(),
        Params {
            chunk_size: 64.into(),
            num_threads: 8.into()
        }
    );
}

#[test]
fn par_fil_auto_when_zero_num_threads() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec.into_par().filter(|x| x > &2).num_threads(0);
    assert_eq!(par.params().num_threads, NumThreads::Auto);
}

#[test]
fn par_fil_auto_when_zero_chunk_size() {
    let vec = vec![1, 4, 2, 1, 5, 6];
    let par = vec.into_par().filter(|x| x <= &2).chunk_size(0);
    assert_eq!(par.params().chunk_size, ChunkSize::Auto);
}

#[test]
fn par_fil_map() {
    let iter = [1, 42, 2, 111, 15, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.into_con_iter().into_par();
    let filter = par.filter(|x| x.len() == 2).num_threads(2);
    let map = filter
        .map(|x| x.parse::<usize>().expect("is-ok"))
        .chunk_size(2);
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &[42, 15]);
}

#[test]
fn par_fil_fmap() {
    let iter = [1, 42, 2, 111, 15, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.into_con_iter().into_par();
    let filter = par.filter(|x| x.len() == 2).num_threads(2);
    let map = filter.flat_map(|x| x.chars().collect::<Vec<_>>());
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &['4', '2', '1', '5']);
}

#[test]
fn par_fil_fmap_option() {
    let iter = [1, 42, 2, 111, 15, 9876].into_iter().skip(1);

    let par = iter.into_con_iter().into_par();
    let filter = par.filter(|x| x.to_string().len() == 2).num_threads(2);
    let map = filter.flat_map(|x| if x % 2 == 1 { Some(x) } else { None });
    let result = map.collect_vec();

    assert_eq!(result.as_slice(), &[15]);
}

#[test]
fn par_fil_filter() {
    let iter = [42, 11, 2, 111, 15, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.into_con_iter().into_par();
    let filter = par.filter(|x| x.len() == 2).num_threads(4);
    let filter = filter
        .filter(|x| x.parse::<usize>().expect("is-ok") % 3 == 0)
        .chunk_size(2);
    let result = filter.collect_vec();

    assert_eq!(result.as_slice(), [15.to_string()]);
}

#[test]
fn par_fil_filtermap() {
    let iter = [42, 11, 2, 111, 15, 9876]
        .map(|x| x.to_string())
        .into_iter()
        .skip(1);

    let par = iter.into_con_iter().into_par();
    let filter = par.filter(|x| x.len() == 2).num_threads(4);
    let filter = filter
        .filter_map(|x| some_if(x, |x| x.parse::<usize>().expect("is-ok") % 3 == 0))
        .chunk_size(2);
    let result = filter.collect_vec();

    assert_eq!(result.as_slice(), [15.to_string()]);
}

// collect

#[test]
fn par_fil_collect() {
    fn test(num_threads: usize, chunk_size: usize) {
        let vec = (54..5448).collect::<Vec<_>>();
        let iter = vec.into_iter().take(10000).into_con_iter().into_par();
        let filter = iter
            .filter(|x| x % 3 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result = filter.collect();

        assert_eq!(result.len(), (5448 - 54) / 3);
        for (i, x) in (54..5448).filter(|x| x % 3 == 0).enumerate() {
            assert_eq!(result.get(i).cloned(), Some(x));
        }
    }
    test_different_params(test)
}

#[test]
fn par_fil_collect_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5448).collect::<Vec<_>>().into_par();
        let filter = iter
            .filter(|x| x % 3 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result = filter.collect_vec();

        assert_eq!(result.len(), (5448 - 54) / 3);
        for (i, x) in (54..5448).filter(|x| x % 3 == 0).enumerate() {
            assert_eq!(result.get(i).cloned(), Some(x));
        }
    }
    test_different_params(test)
}

#[test]
fn par_fil_collect_into() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5448).collect::<Vec<_>>().into_par();
        let map = iter
            .filter(|x| x % 3 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result1 = map.collect_into(SplitVec::new());

        assert_eq!(result1.len(), (5448 - 54) / 3);
        for (i, x) in (54..5448).filter(|x| x % 3 == 0).enumerate() {
            assert_eq!(result1.get(i).cloned(), Some(x));
        }

        let iter = (5448..6000).collect::<Vec<_>>().into_par();
        let map = iter
            .filter(|x| x % 3 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result2 = map.collect_into(result1);
        assert_eq!(result2.len(), (6000 - 54) / 3);
        for (i, x) in (54..6000).filter(|x| x % 3 == 0).enumerate() {
            assert_eq!(result2.get(i).cloned(), Some(x));
        }
    }
    test_different_params(test)
}

#[test]
fn par_fil_collect_into_vec() {
    fn test(num_threads: usize, chunk_size: usize) {
        let iter = (54..5448).collect::<Vec<_>>().into_par();
        let filter = iter
            .filter(|x| x % 3 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result1 = filter.collect_into(vec![]);

        assert_eq!(result1.len(), (5448 - 54) / 3);
        for (i, x) in (54..5448).filter(|x| x % 3 == 0).enumerate() {
            assert_eq!(result1.get(i).cloned(), Some(x));
        }

        let iter = (5448..6000).collect::<Vec<_>>().into_par();
        let filter = iter
            .filter(|x| x % 3 == 0)
            .num_threads(num_threads)
            .chunk_size(chunk_size);
        let result2 = filter.collect_into(result1);
        assert_eq!(result2.len(), (6000 - 54) / 3);
        for (i, x) in (54..6000).filter(|x| x % 3 == 0).enumerate() {
            assert_eq!(result2.get(i).cloned(), Some(x));
        }
    }

    test_different_params(test)
}

// count

#[test]
fn par_fil_count() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.filter(|x| x % 3 == 2);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.count(), (13..4785).filter(|x| x % 3 == 2).count());
    }
    test_different_params(test)
}

// find

#[test]
fn par_fil_next() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.filter(|x| x % 312 == 0);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.first_with_index(), Some((312 - 13, 312)));
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (0..0).into_con_iter().into_par();
        let par = par.filter(|x| x % 312 == 0);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.first_with_index(), None);
    }
    test_different_params(test_empty);
}

#[test]
fn par_fil_find() {
    fn test(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.filter(|x| x % 312 == 0);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.find_with_index(|x| x >= &500), Some((624 - 13, 624)));
    }
    test_different_params(test);

    fn test_empty(num_threads: usize, chunk_size: usize) {
        let par = (13..4785).into_con_iter().into_par();
        let par = par.filter(|x| x % 312 == 0);
        let par = par.num_threads(num_threads).chunk_size(chunk_size);
        assert_eq!(par.find_with_index(|x| x > &13333), None);
    }
    test_different_params(test_empty);
}

// reduce

#[test]
fn par_fil_reduce_i64() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (-(len as i64)..(2 * len as i64))
                .collect::<Vec<_>>()
                .into_par()
                .filter(|x| *x >= 0 && *x < len as i64)
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
fn par_fil_reduce_string() {
    fn test(len: usize, num_threads: usize, chunk_size: usize) {
        let data = || {
            (-(len as i64)..(2 * len as i64))
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .into_par()
                .filter(|x| {
                    let x: i64 = x.parse().expect("is-ok");
                    x >= 0 && x < len as i64
                })
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
