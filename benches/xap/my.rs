/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ll/iter/1024        time:   [3.7371 µs 3.7553 µs 3.7739 µs]
xap_ll/xap/1024         time:   [10.224 µs 10.290 µs 10.356 µs]

xap_ll/iter/32768       time:   [120.36 µs 120.99 µs 121.62 µs]
xap_ll/xap/32768        time:   [332.57 µs 334.81 µs 337.06 µs]

xap_ll/iter/1048576     time:   [3.8710 ms 3.8878 ms 3.9046 ms]
xap_ll/xap/1048576      time:   [10.678 ms 10.741 ms 10.807 ms]


COLLECT:
xap_ll/iter/1024        time:   [4.1939 µs 4.2358 µs 4.2788 µs]
xap_ll/xap/1024         time:   [37.790 µs 38.423 µs 39.039 µs]

xap_ll/iter/32768       time:   [192.32 µs 194.13 µs 196.18 µs]
xap_ll/xap/32768        time:   [1.1706 ms 1.1917 ms 1.2127 ms]

xap_ll/iter/1048576     time:   [65.585 ms 66.152 ms 66.795 ms]
xap_ll/xap/1048576      time:   [95.022 ms 96.462 ms 98.320 ms]

(!) significant difference

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{
    Id, Xap, XapCopied, count::iter::FlatMapIterMany, fun::flat_map::FnFlatMap,
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f1_v(i: u64) -> impl IntoIterator<Item = u64> {
    (0..2).map(move |x| x + i + 1).collect::<Vec<_>>()
}

fn f2_v(i: u64) -> impl IntoIterator<Item = u64> {
    (0..2).map(move |x| i * 7 + x).collect::<Vec<_>>()
}

fn f1_i(i: u64) -> impl IntoIterator<Item = u64> {
    (0..2).map(move |x| x + i + 1).filter(|x| *x < 1 << 20)
}

fn f2_i(i: u64) -> impl IntoIterator<Item = u64> {
    (0..2).map(move |x| i * 7 + x).filter(|x| *x < 1 << 20)
}

fn f1_c(i: u64) -> [u64; 2] {
    [i + 1 + 0, i + 1 + 1]
}

fn f2_c(i: u64) -> [u64; 2] {
    // [i * 7 + 0, i * 7 + 1, i * 7 + 2, i * 7 + 3, i * 7 + 4]
    [i * 7 + 0, i * 7 + 1]
}

fn iter_v(inputs: &[u64]) -> Vec<u64> {
    let it = inputs.iter().copied().flat_map(f1_v).flat_map(f2_v);
    // return it.collect();
    let mut v = vec![];
    for x in it {
        v.push(x);
    }
    v
}

fn iter_my_v(inputs: &[u64]) -> Vec<u64> {
    let it = inputs.iter().copied().flat_map(|x| [x]);
    let it = FlatMapIterMany::new(it, FnFlatMap::new(f1_v));
    let it = FlatMapIterMany::new(it, FnFlatMap::new(f2_v));
    // return it.collect();
    let mut v = vec![];
    for x in it {
        v.push(x);
    }
    v
}

fn xap_v(inputs: &[u64]) -> Vec<u64> {
    let xap = Id::new().copied().flat_map(f1_v).flat_map(f2_v);
    let it = inputs.iter().flat_map(|x| xap.xap(x));
    // return it.collect();
    let mut v = vec![];
    for x in it {
        v.push(x);
    }
    v
}

fn iter_i(inputs: &[u64]) -> Vec<u64> {
    let it = inputs.iter().copied().flat_map(f1_i).flat_map(f2_i);
    // return it.collect();
    let mut v = vec![];
    for x in it {
        v.push(x);
    }
    v
}

fn iter_my_i(inputs: &[u64]) -> Vec<u64> {
    let it = inputs.iter().copied().flat_map(|x| [x]);
    let it = FlatMapIterMany::new(it, FnFlatMap::new(f1_i));
    let it = FlatMapIterMany::new(it, FnFlatMap::new(f2_i));
    // return it.collect();
    let mut v = vec![];
    for x in it {
        v.push(x);
    }
    v
}

fn xap_i(inputs: &[u64]) -> Vec<u64> {
    let xap = Id::new().copied().flat_map(f1_i).flat_map(f2_i);
    let it = inputs.iter().flat_map(|x| xap.xap(x));
    // return it.collect();
    let mut v = vec![];
    for x in it {
        v.push(x);
    }
    v
}

fn iter_c(inputs: &[u64]) -> Vec<u64> {
    let it = inputs.iter().copied().flat_map(f1_c).flat_map(f2_c);
    // return it.collect();
    let mut v = vec![];
    for x in it {
        v.push(x);
    }
    v
}

fn iter_my_c(inputs: &[u64]) -> Vec<u64> {
    let it = inputs.iter().copied().flat_map(|x| [x]);
    let it = FlatMapIterMany::new(it, FnFlatMap::new(f1_c));
    let it = FlatMapIterMany::new(it, FnFlatMap::new(f2_c));
    // return it.collect();
    let mut v = vec![];
    for x in it {
        v.push(x);
    }
    v
}

fn xap_c(inputs: &[u64]) -> Vec<u64> {
    let xap = Id::new().copied().flat_map(f1_c).flat_map(f2_c);
    let it = inputs.iter().flat_map(|x| xap.xap(x));
    // return it.collect();
    let mut v = vec![];
    for x in it {
        v.push(x);
    }
    v
}

fn run(c: &mut Criterion) {
    let len = [1 << 12, 1 << 15, 1 << 17];

    let mut group = c.benchmark_group("my");

    for n in len {
        let input = inputs(n);
        let expected = iter_v(&input);

        group.bench_with_input(BenchmarkId::new("iter_v", n), &n, |b, _| {
            assert_eq!(&expected, &iter_v(&input));
            b.iter(|| iter_v(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("iter_my_v", n), &n, |b, _| {
            assert_eq!(&expected, &iter_my_v(&input));
            b.iter(|| iter_my_v(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("xap_v", n), &n, |b, _| {
            assert_eq!(&expected, &xap_v(&input));
            b.iter(|| xap_v(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("iter_i", n), &n, |b, _| {
            assert_eq!(&expected, &iter_i(&input));
            b.iter(|| iter_i(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("iter_my_i", n), &n, |b, _| {
            assert_eq!(&expected, &iter_my_i(&input));
            b.iter(|| iter_my_i(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("xap_i", n), &n, |b, _| {
            assert_eq!(&expected, &xap_i(&input));
            b.iter(|| xap_i(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("iter_c", n), &n, |b, _| {
            assert_eq!(&expected, &iter_c(&input));
            b.iter(|| iter_c(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("iter_my_c", n), &n, |b, _| {
            assert_eq!(&expected, &iter_my_c(&input));
            b.iter(|| iter_my_c(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("xap_c", n), &n, |b, _| {
            assert_eq!(&expected, &xap_c(&input));
            b.iter(|| xap_c(black_box(&input)))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
