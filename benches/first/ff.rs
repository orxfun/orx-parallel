/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used
* beg & mid & end show where the element to be found is located

first_ff/seq/e20_early_light        time:   [145.92 ns 147.45 ns 149.10 ns]
first_ff/rayon/e20_early_light      time:   [3.2938 ms 3.5839 ms 3.9020 ms]
first_ff/orx/e20_early_light        time:   [1.8578 ms 2.0344 ms 2.2415 ms]

first_ff/seq/e20_mid_light          time:   [366.85 µs 377.19 µs 389.72 µs]
first_ff/rayon/e20_mid_light        time:   [20.844 ms 22.264 ms 23.746 ms]
first_ff/orx/e20_mid_light          time:   [3.4134 ms 3.6214 ms 3.8502 ms]

first_ff/seq/e20_late_light         time:   [822.92 µs 837.09 µs 853.30 µs]
first_ff/rayon/e20_late_light       time:   [25.034 ms 30.663 ms 39.571 ms]
first_ff/orx/e20_late_light         time:   [3.7699 ms 4.0878 ms 4.4360 ms]

first_ff/seq/e20_early_heavy        time:   [1.6255 µs 1.6495 µs 1.6721 µs]
first_ff/rayon/e20_early_heavy      time:   [3.4891 ms 3.9140 ms 4.4395 ms]
first_ff/orx/e20_early_heavy        time:   [3.0102 ms 3.3720 ms 3.8170 ms]

first_ff/seq/e20_mid_heavy          time:   [4.5670 ms 4.6294 ms 4.6943 ms]
first_ff/rayon/e20_mid_heavy        time:   [20.752 ms 24.542 ms 29.477 ms]
first_ff/orx/e20_mid_heavy          time:   [5.5180 ms 6.1753 ms 6.8832 ms]

first_ff/seq/e20_late_heavy         time:   [8.9405 ms 9.0422 ms 9.1462 ms]
first_ff/rayon/e20_late_heavy       time:   [18.929 ms 20.395 ms 21.948 ms]
first_ff/orx/e20_late_heavy         time:   [4.2351 ms 4.5642 ms 4.9639 ms]

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
    match h {
        true => input
            .into_par()
            .filter(|x| x.is_multiple_of(9))
            .filter(|x| h_f(**x, value))
            .first()
            .copied(),
        false => input
            .into_par()
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

    let mut group = c.benchmark_group("first_ff");

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
