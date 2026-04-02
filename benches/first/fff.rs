/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used
* beg & mid & end show where the element to be found is located

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
use orx_parallel::infallible::par;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hint::black_box;

fn inputs(len: usize, pos: usize, val: u64) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut vec = Vec::with_capacity(len);
    vec.extend((0..(len - 1)).map(|_| rng.random_range(0..150)));
    vec.insert(pos, val);
    vec
}

const FIB_UPPER_BOUND: u64 = 201;

fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn l_f(x: u64, value: u64) -> bool {
    x == value
}

fn h_f(x: u64, value: u64) -> bool {
    let a = black_box(fibonacci(x % FIB_UPPER_BOUND));
    let b = black_box(fibonacci(x % FIB_UPPER_BOUND));
    a - b + x == value
}

fn seq(input: &[u64], value: u64, h: bool) -> Option<u64> {
    match h {
        true => input
            .iter()
            .filter(|x| h_f(**x, value))
            .filter(|x| *x + 1 > 900)
            .filter(|x| x.is_multiple_of(9))
            .next()
            .copied(),
        false => input
            .iter()
            .filter(|x| l_f(**x, value))
            .filter(|x| *x + 1 > 900)
            .filter(|x| x.is_multiple_of(9))
            .next()
            .copied(),
    }
}

fn orx(input: &[u64], value: u64, h: bool) -> Option<u64> {
    let iter = input.into_con_iter();
    match h {
        true => par(iter)
            .filter(|x| h_f(**x, value))
            .filter(|x| *x + 1 > 900)
            .filter(|x| x.is_multiple_of(9))
            .first()
            .copied(),
        false => par(iter)
            .filter(|x| l_f(**x, value))
            .filter(|x| *x + 1 > 900)
            .filter(|x| x.is_multiple_of(9))
            .first()
            .copied(),
    }
}

fn rayon(input: &[u64], value: u64, h: bool) -> Option<u64> {
    match h {
        true => input
            .into_par_iter()
            .filter(|x| h_f(**x, value))
            .filter(|x| *x + 1 > 900)
            .filter(|x| x.is_multiple_of(9))
            .find_first(|_| true)
            .copied(),
        false => input
            .into_par_iter()
            .filter(|x| l_f(**x, value))
            .filter(|x| *x + 1 > 900)
            .filter(|x| x.is_multiple_of(9))
            .find_first(|_| true)
            .copied(),
    }
}

struct Treat {
    len: usize,
    pos: usize,
    val: u64,
    name: String,
    heavy_compute: bool,
}

fn run(c: &mut Criterion) {
    let treatments = [
        Treat {
            len: 1 << 20,
            pos: 1 << 8,
            name: format!("e{}_early_light", 20),
            val: 999,
            heavy_compute: false,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 19) + 7,
            name: format!("e{}_mid_light", 20),
            val: 999,
            heavy_compute: false,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 20) - 27,
            name: format!("e{}_late_light", 20),
            val: 999,
            heavy_compute: false,
        },
        Treat {
            len: 1 << 20,
            pos: 1 << 8,
            name: format!("e{}_early_heavy", 20),
            val: 999,
            heavy_compute: true,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 19) + 7,
            name: format!("e{}_mid_heavy", 20),
            val: 999,
            heavy_compute: true,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 20) - 27,
            name: format!("e{}_late_heavy", 20),
            val: 999,
            heavy_compute: true,
        },
    ];

    let mut group = c.benchmark_group("first_fff");

    for t in treatments {
        let input = inputs(t.len, t.pos, t.val);
        let expected = seq(&input, t.val, t.heavy_compute);

        group.bench_with_input(BenchmarkId::new("seq", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.val, t.heavy_compute));
            b.iter(|| seq(&input, t.val, t.heavy_compute))
        });

        group.bench_with_input(BenchmarkId::new("rayon", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &rayon(&input, t.val, t.heavy_compute));
            b.iter(|| rayon(&input, t.val, t.heavy_compute))
        });

        group.bench_with_input(BenchmarkId::new("orx", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &orx(&input, t.val, t.heavy_compute));
            b.iter(|| orx(&input, t.val, t.heavy_compute))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
