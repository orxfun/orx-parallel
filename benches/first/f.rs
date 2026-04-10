/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used
* beg & mid & end show where the element to be found is located

first_f/seq/e20_early_light     time:   [134.43 ns 135.65 ns 137.10 ns]
first_f/rayon/e20_early_light   time:   [3.0049 ms 3.1196 ms 3.2361 ms]
first_f/orx/e20_early_light     time:   [2.3006 ms 2.8179 ms 3.5227 ms]

first_f/seq/e20_mid_light       time:   [254.82 µs 256.43 µs 258.11 µs]
first_f/rayon/e20_mid_light     time:   [16.069 ms 16.768 ms 17.485 ms]
first_f/orx/e20_mid_light       time:   [2.5330 ms 2.5800 ms 2.6287 ms]

first_f/seq/e20_late_light      time:   [549.48 µs 553.65 µs 557.91 µs]
first_f/rayon/e20_late_light    time:   [18.432 ms 19.354 ms 20.327 ms]
first_f/orx/e20_late_light      time:   [2.6065 ms 2.6594 ms 2.7185 ms]

first_f/seq/e20_early_heavy     time:   [9.1386 µs 9.1968 µs 9.2558 µs]
first_f/rayon/e20_early_heavy   time:   [3.3045 ms 3.5344 ms 3.7845 ms]
first_f/orx/e20_early_heavy     time:   [2.3561 ms 2.4414 ms 2.5357 ms]

first_f/seq/e20_mid_heavy       time:   [29.245 ms 29.492 ms 29.754 ms]
first_f/rayon/e20_mid_heavy     time:   [16.331 ms 17.904 ms 19.619 ms]
first_f/orx/e20_mid_heavy       time:   [6.8600 ms 7.0786 ms 7.3210 ms]

first_f/seq/e20_late_heavy      time:   [62.792 ms 64.373 ms 66.394 ms]
first_f/rayon/e20_late_heavy    time:   [18.846 ms 19.785 ms 20.808 ms]
first_f/orx/e20_late_heavy      time:   [11.166 ms 11.519 ms 11.883 ms]
*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
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
        true => input.iter().filter(|x| h_f(**x, value)).next().copied(),
        false => input.iter().filter(|x| l_f(**x, value)).next().copied(),
    }
}

fn orx(input: &[u64], value: u64, h: bool) -> Option<u64> {
    match h {
        true => input.into_par().filter(|x| h_f(**x, value)).copied(),
        false => input
            .into_par()
            .filter(|x| l_f(**x, value))
            .first()
            .copied(),
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
    heavy: bool,
}

fn run(c: &mut Criterion) {
    let treatments = [
        Treat {
            len: 1 << 20,
            pos: 1 << 8,
            name: format!("e{}_early_light", 20),
            val: 999,
            heavy: false,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 19) + 7,
            name: format!("e{}_mid_light", 20),
            val: 999,
            heavy: false,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 20) - 27,
            name: format!("e{}_late_light", 20),
            val: 999,
            heavy: false,
        },
        Treat {
            len: 1 << 20,
            pos: 1 << 8,
            name: format!("e{}_early_heavy", 20),
            val: 999,
            heavy: true,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 19) + 7,
            name: format!("e{}_mid_heavy", 20),
            val: 999,
            heavy: true,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 20) - 27,
            name: format!("e{}_late_heavy", 20),
            val: 999,
            heavy: true,
        },
    ];

    let mut group = c.benchmark_group("first_f");

    for t in treatments {
        let input = inputs(t.len, t.pos, t.val);
        let expected = seq(&input, t.val, t.heavy);

        group.bench_with_input(BenchmarkId::new("seq", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.val, t.heavy));
            b.iter(|| seq(&input, t.val, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("rayon", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &rayon(&input, t.val, t.heavy));
            b.iter(|| rayon(&input, t.val, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("orx", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &orx(&input, t.val, t.heavy));
            b.iter(|| orx(&input, t.val, t.heavy))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
