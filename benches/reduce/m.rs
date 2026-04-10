/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used

reduce_m/seq/e15_light      time:   [14.223 µs 14.358 µs 14.497 µs]
reduce_m/rayon1/e15_light   time:   [8.7471 ms 8.8833 ms 9.0200 ms]
reduce_m/rayon2/e15_light   time:   [9.9737 ms 10.162 ms 10.351 ms]
reduce_m/orx/e15_light      time:   [1.4237 ms 1.4667 ms 1.5136 ms]

reduce_m/seq/e20_light      time:   [638.87 µs 657.68 µs 683.73 µs]
reduce_m/rayon1/e20_light   time:   [19.552 ms 20.340 ms 21.125 ms]
reduce_m/rayon2/e20_light   time:   [19.648 ms 20.201 ms 20.775 ms]
reduce_m/orx/e20_light      time:   [2.3489 ms 2.4457 ms 2.5619 ms]

reduce_m/seq/e15_heavy      time:   [1.7356 ms 1.7860 ms 1.8412 ms]
reduce_m/rayon1/e15_heavy   time:   [10.757 ms 11.075 ms 11.379 ms]
reduce_m/rayon2/e15_heavy   time:   [10.851 ms 11.165 ms 11.467 ms]
reduce_m/orx/e15_heavy      time:   [2.2275 ms 2.2514 ms 2.2761 ms]

reduce_m/seq/e20_heavy      time:   [50.382 ms 51.004 ms 51.678 ms]
reduce_m/rayon1/e20_heavy   time:   [10.160 ms 10.875 ms 11.679 ms]
reduce_m/rayon2/e20_heavy   time:   [9.9150 ms 10.458 ms 11.015 ms]
reduce_m/orx/e20_heavy      time:   [7.7945 ms 7.9175 ms 8.0515 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hint::black_box;

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

const FIB_UPPER_BOUND: u64 = 301;

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

fn l_r(a: u64, b: u64) -> u64 {
    a + b
}

fn h_r(a: u64, b: u64) -> u64 {
    let f = black_box(fibonacci(a % FIB_UPPER_BOUND));
    let g = black_box(a + f);
    g + b - f
}

fn m(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

fn seq(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.iter().map(m).reduce(h_r),
        false => input.iter().map(m).reduce(l_r),
    }
}

fn orx(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.into_par().map(m).reduce(h_r),
        false => input.into_par().map(m).reduce(l_r),
    }
}

fn rayon1(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.into_par_iter().map(m).reduce_with(h_r),
        false => input.into_par_iter().map(m).reduce_with(l_r),
    }
}

fn rayon2(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => Some(input.into_par_iter().map(m).reduce(|| 0, h_r)),
        false => Some(input.into_par_iter().map(m).reduce(|| 0, l_r)),
    }
}

struct Treat {
    len: usize,
    heavy: bool,
}

fn run(c: &mut Criterion) {
    let treatments = [
        Treat {
            len: 1 << 15,
            heavy: false,
        },
        Treat {
            len: 1 << 20,
            heavy: false,
        },
        Treat {
            len: 1 << 15,
            heavy: true,
        },
        Treat {
            len: 1 << 20,
            heavy: true,
        },
    ];

    let mut group = c.benchmark_group("reduce_m");

    for t in treatments {
        let name = format!(
            "e{}_{}",
            t.len.ilog2(),
            match t.heavy {
                true => "heavy",
                false => "light",
            },
        );
        let input = inputs(t.len);
        let expected = seq(&input, t.heavy);

        group.bench_with_input(BenchmarkId::new("seq", &name), &name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.heavy));
            b.iter(|| seq(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("rayon1", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon1(&input, t.heavy));
            b.iter(|| rayon1(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("rayon2", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon2(&input, t.heavy));
            b.iter(|| rayon2(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("orx", &name), &name, |b, _| {
            assert_eq!(&expected, &orx(&input, t.heavy));
            b.iter(|| orx(&input, t.heavy))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
