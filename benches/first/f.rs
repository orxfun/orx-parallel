/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used
* beg & mid & end show where the element to be found is located

first_f/seq/e20_early   time:   [141.21 ns 142.09 ns 143.01 ns]
first_f/rayon/e20_early time:   [3.5811 ms 3.7413 ms 3.9083 ms]
first_f/orx/e20_early   time:   [1.6906 ms 1.7142 ms 1.7387 ms]

first_f/seq/e20_mid     time:   [282.47 µs 291.11 µs 302.48 µs]
first_f/rayon/e20_mid   time:   [21.074 ms 22.538 ms 24.097 ms]
first_f/orx/e20_mid     time:   [3.0550 ms 3.1318 ms 3.2164 ms]

first_f/seq/e20_late    time:   [623.79 µs 642.33 µs 665.24 µs]
first_f/rayon/e20_late  time:   [21.120 ms 22.075 ms 23.059 ms]
first_f/orx/e20_late    time:   [3.1072 ms 3.2949 ms 3.5124 ms]
*/

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::IntoConcurrentIter;
use orx_parallel::infallible::par;
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
        true => input.iter().filter(|x| h_f(**x, value)).next().copied(),
        false => input.iter().filter(|x| l_f(**x, value)).next().copied(),
    }
}

fn orx(input: &[u64], value: u64, h: bool) -> Option<u64> {
    let iter = input.into_con_iter();
    match h {
        true => par(iter).filter(|x| h_f(**x, value)).first().copied(),
        false => par(iter).filter(|x| l_f(**x, value)).first().copied(),
    }
}

fn rayon(input: &[u64], value: u64, h: bool) -> Option<u64> {
    match h {
        true => input
            .into_par_iter()
            .filter(|x| h_f(**x, value))
            .find_first(|_| true)
            .copied(),
        false => input
            .into_par_iter()
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

    let mut group = c.benchmark_group("first_f");

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
