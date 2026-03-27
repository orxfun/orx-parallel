/*

* light & heavy show the intensity of computation

first_fff/seq/e20_early  time:   [120.17 ns 121.15 ns 122.19 ns]
first_fff/rayon/e20_earlytime:   [2.7752 ms 2.8812 ms 2.9894 ms]
first_fff/orx/e20_early  time:   [1.4441 ms 1.5078 ms 1.5780 ms]

first_fff/seq/e20_mid    time:   [266.75 µs 271.41 µs 276.84 µs]
first_fff/rayon/e20_mid  time:   [17.954 ms 18.922 ms 20.099 ms]
first_fff/orx/e20_mid    time:   [2.0867 ms 2.1164 ms 2.1480 ms]

first_fff/seq/e20_late   time:   [526.36 µs 532.63 µs 539.28 µs]
first_fff/rayon/e20_late time:   [8.6408 ms 9.6961 ms 10.784 ms]
first_fff/orx/e20_late   time:   [2.3913 ms 2.4260 ms 2.4619 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::IntoConcurrentIter;
use orx_parallel::par;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn inputs(len: usize, pos: usize, val: u64) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut vec = Vec::with_capacity(len);
    vec.extend((0..(len - 1)).map(|_| rng.random_range(0..150)));
    vec.insert(pos, val);
    vec
}

fn seq(input: &[u64], value: u64) -> Option<u64> {
    input
        .iter()
        .filter(|x| x.is_multiple_of(9))
        .filter(|x| *x + 1 > 900)
        .filter(|x| **x == value)
        .next()
        .copied()
}

fn orx(input: &[u64], value: u64) -> Option<u64> {
    let iter = input.into_con_iter();
    par(iter)
        .filter(|x| x.is_multiple_of(9))
        .filter(|x| *x + 1 > 900)
        .filter(|x| **x == value)
        .first()
        .copied()
}

fn rayon(input: &[u64], value: u64) -> Option<u64> {
    input
        .into_par_iter()
        .filter(|x| x.is_multiple_of(9))
        .filter(|x| *x + 1 > 900)
        .filter(|x| **x == value)
        .find_first(|_| true)
        .copied()
}

struct Treat {
    len: usize,
    pos: usize,
    val: u64,
    name: String,
}

fn run(c: &mut Criterion) {
    let treatments = [
        Treat {
            len: 1 << 20,
            pos: 1 << 8,
            name: format!("e{}_early", 20),
            val: 999,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 19) + 7,
            name: format!("e{}_mid", 20),
            val: 999,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 20) - 27,
            name: format!("e{}_late", 20),
            val: 999,
        },
    ];

    let mut group = c.benchmark_group("first_fff");

    for t in treatments {
        let input = inputs(t.len, t.pos, t.val);
        let expected = seq(&input, t.val);

        group.bench_with_input(BenchmarkId::new("seq", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.val));
            b.iter(|| seq(&input, t.val))
        });

        group.bench_with_input(BenchmarkId::new("rayon", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &rayon(&input, t.val));
            b.iter(|| rayon(&input, t.val))
        });

        group.bench_with_input(BenchmarkId::new("orx", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &orx(&input, t.val));
            b.iter(|| orx(&input, t.val))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
