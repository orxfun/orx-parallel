/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used

reduce_m/seq/e15_light      time:   [11.074 µs 11.161 µs 11.247 µs]
reduce_m/rayon1/e15_light   time:   [7.0328 ms 7.1301 ms 7.2289 ms]
reduce_m/rayon2/e15_light   time:   [2.9261 ms 3.5955 ms 4.2788 ms]
reduce_m/orx/e15_light      time:   [1.1085 ms 1.1392 ms 1.1707 ms]

reduce_m/seq/e20_light      time:   [495.38 µs 502.32 µs 509.44 µs]
reduce_m/rayon1/e20_light   time:   [13.399 ms 13.979 ms 14.505 ms]
reduce_m/rayon2/e20_light   time:   [13.502 ms 14.247 ms 14.951 ms]
reduce_m/orx/e20_light      time:   [1.8877 ms 1.9350 ms 1.9825 ms]

reduce_m/seq/e15_heavy      time:   [1.2836 ms 1.2937 ms 1.3049 ms]
reduce_m/rayon1/e15_heavy   time:   [7.8716 ms 8.0846 ms 8.2734 ms]
reduce_m/rayon2/e15_heavy   time:   [8.9437 ms 9.1293 ms 9.3129 ms]
reduce_m/orx/e15_heavy      time:   [1.8329 ms 1.8510 ms 1.8711 ms]

reduce_m/seq/e20_heavy      time:   [40.288 ms 40.537 ms 40.796 ms]
reduce_m/rayon1/e20_heavy   time:   [7.5214 ms 8.0231 ms 8.5305 ms]
reduce_m/rayon2/e20_heavy   time:   [7.5114 ms 7.9549 ms 8.4085 ms]
reduce_m/orx/e20_heavy      time:   [5.9074 ms 5.9774 ms 6.0528 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::IntoConcurrentIter;
use orx_parallel::infallible::par;
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
        true => par(input.into_con_iter()).map(m).reduce(h_r),
        false => par(input.into_con_iter()).map(m).reduce(l_r),
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
    heavy_compute: bool,
}

fn run(c: &mut Criterion) {
    let treatments = [
        Treat {
            len: 1 << 15,
            heavy_compute: false,
        },
        Treat {
            len: 1 << 20,
            heavy_compute: false,
        },
        Treat {
            len: 1 << 15,
            heavy_compute: true,
        },
        Treat {
            len: 1 << 20,
            heavy_compute: true,
        },
    ];

    let mut group = c.benchmark_group("reduce_m");

    for t in treatments {
        let name = format!(
            "e{}_{}",
            t.len.ilog2(),
            match t.heavy_compute {
                true => "heavy",
                false => "light",
            },
        );
        let input = inputs(t.len);
        let expected = seq(&input, t.heavy_compute);

        group.bench_with_input(BenchmarkId::new("seq", &name), &name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.heavy_compute));
            b.iter(|| seq(&input, t.heavy_compute))
        });

        group.bench_with_input(BenchmarkId::new("rayon1", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon1(&input, t.heavy_compute));
            b.iter(|| rayon1(&input, t.heavy_compute))
        });

        group.bench_with_input(BenchmarkId::new("rayon2", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon2(&input, t.heavy_compute));
            b.iter(|| rayon2(&input, t.heavy_compute))
        });

        group.bench_with_input(BenchmarkId::new("orx", &name), &name, |b, _| {
            assert_eq!(&expected, &orx(&input, t.heavy_compute));
            b.iter(|| orx(&input, t.heavy_compute))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
