/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used
* beg & mid & end show where the element to be found is located

first_fff/seq/e20_early_light   time:   [262.80 ns 265.39 ns 268.11 ns]
first_fff/rayon/e20_early_light time:   [3.2993 ms 3.4595 ms 3.6258 ms]
first_fff/orx/e20_early_light   time:   [2.1888 ms 2.3173 ms 2.4550 ms]

first_fff/seq/e20_mid_light     time:   [1.4297 ms 1.4540 ms 1.4811 ms]
first_fff/rayon/e20_mid_light   time:   [16.840 ms 18.066 ms 19.320 ms]
first_fff/orx/e20_mid_light     time:   [3.4804 ms 3.6483 ms 3.8281 ms]

first_fff/seq/e20_late_light    time:   [2.7239 ms 2.7541 ms 2.7844 ms]
first_fff/rayon/e20_late_light  time:   [20.120 ms 21.524 ms 23.026 ms]
first_fff/orx/e20_late_light    time:   [4.2533 ms 4.6806 ms 5.1731 ms]

first_fff/seq/e20_early_heavy   time:   [10.086 µs 10.276 µs 10.473 µs]
first_fff/rayon/e20_early_heavy time:   [3.2627 ms 3.4016 ms 3.5408 ms]
first_fff/orx/e20_early_heavy   time:   [2.0236 ms 2.0808 ms 2.1420 ms]

first_fff/seq/e20_mid_heavy     time:   [29.877 ms 30.286 ms 30.744 ms]
first_fff/rayon/e20_mid_heavy   time:   [21.912 ms 25.140 ms 29.709 ms]
first_fff/orx/e20_mid_heavy     time:   [7.0121 ms 7.2658 ms 7.5417 ms]

first_fff/seq/e20_late_heavy    time:   [59.441 ms 60.155 ms 60.856 ms]
first_fff/rayon/e20_late_heavy  time:   [16.443 ms 17.635 ms 19.247 ms]
first_fff/orx/e20_late_heavy    time:   [9.6193 ms 10.580 ms 11.931 ms]

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
    match h {
        true => input
            .into_par()
            .filter(|x| h_f(**x, value))
            .filter(|x| *x + 1 > 900)
            .filter(|x| x.is_multiple_of(9))
            .first()
            .copied(),
        false => input
            .into_par()
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

    let mut group = c.benchmark_group("first_fff");

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
