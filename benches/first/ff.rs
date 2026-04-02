/*

* beg & mid & end show where the element to be found is located

first_ff/seq/e20_early  time:   [166.39 ns 168.56 ns 170.79 ns]
first_ff/rayon/e20_earlytime:   [2.4113 ms 2.4959 ms 2.5849 ms]
first_ff/orx/e20_early  time:   [1.3151 ms 1.3346 ms 1.3564 ms]

first_ff/seq/e20_mid    time:   [243.25 µs 245.68 µs 248.09 µs]
first_ff/rayon/e20_mid  time:   [15.726 ms 16.167 ms 16.614 ms]
first_ff/orx/e20_mid    time:   [2.0945 ms 2.1235 ms 2.1544 ms]

first_ff/seq/e20_late   time:   [514.54 µs 522.25 µs 530.53 µs]
first_ff/rayon/e20_late time:   [17.518 ms 18.162 ms 18.783 ms]
first_ff/orx/e20_late   time:   [2.1841 ms 2.2082 ms 2.2344 ms]

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
            .filter(|x| x.is_multiple_of(9))
            .filter(|x| h_f(**x, value))
            .next()
            .copied(),
        false => input
            .iter()
            .filter(|x| x.is_multiple_of(9))
            .filter(|x| l_f(**x, value))
            .next()
            .copied(),
    }
}

fn orx(input: &[u64], value: u64, h: bool) -> Option<u64> {
    let iter = input.into_con_iter();
    match h {
        true => par(iter)
            .filter(|x| x.is_multiple_of(9))
            .filter(|x| h_f(**x, value))
            .first()
            .copied(),
        false => par(iter)
            .filter(|x| x.is_multiple_of(9))
            .filter(|x| l_f(**x, value))
            .first()
            .copied(),
    }
}

fn rayon(input: &[u64], value: u64, h: bool) -> Option<u64> {
    match h {
        true => input
            .into_par_iter()
            .filter(|x| x.is_multiple_of(9))
            .filter(|x| h_f(**x, value))
            .find_first(|_| true)
            .copied(),
        false => input
            .into_par_iter()
            .filter(|x| x.is_multiple_of(9))
            .filter(|x| l_f(**x, value))
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

    let mut group = c.benchmark_group("first_ff");

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
