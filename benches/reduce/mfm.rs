/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used

reduce_mf/seq/e15_light     time:   [24.474 µs 24.698 µs 24.935 µs]
reduce_mf/rayon1/e15_light  time:   [8.1422 ms 8.2820 ms 8.4060 ms]
reduce_mf/rayon2/e15_light  time:   [7.3712 ms 7.8117 ms 8.2125 ms]
reduce_mf/orx/e15_light     time:   [1.1941 ms 1.2191 ms 1.2482 ms]

reduce_mf/seq/e20_light     time:   [964.37 µs 976.02 µs 989.32 µs]
reduce_mf/rayon1/e20_light  time:   [15.169 ms 15.627 ms 16.098 ms]
reduce_mf/rayon2/e20_light  time:   [15.516 ms 15.902 ms 16.291 ms]
reduce_mf/orx/e20_light     time:   [1.8821 ms 1.9049 ms 1.9285 ms]

reduce_mf/seq/e15_heavy     time:   [1.2139 ms 1.2220 ms 1.2312 ms]
reduce_mf/rayon1/e15_heavy  time:   [6.4007 ms 7.0719 ms 7.7221 ms]
reduce_mf/rayon2/e15_heavy  time:   [9.6266 ms 9.8521 ms 10.058 ms]
reduce_mf/orx/e15_heavy     time:   [2.0351 ms 2.0709 ms 2.1092 ms]

reduce_mf/seq/e20_heavy     time:   [40.212 ms 40.674 ms 41.176 ms]
reduce_mf/rayon1/e20_heavy  time:   [5.6430 ms 6.0319 ms 6.4429 ms]
reduce_mf/rayon2/e20_heavy  time:   [7.0000 ms 7.5254 ms 8.0761 ms]
reduce_mf/orx/e20_heavy     time:   [6.0235 ms 6.0753 ms 6.1275 ms]

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

fn f(a: &u64) -> bool {
    !(a + 7).is_multiple_of(11)
}

fn h_m2(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => fibonacci(n % FIB_UPPER_BOUND) + 1000,
    }
}

fn l_m2(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

fn seq(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.iter().map(m).filter(f).reduce(h_r),
        false => input.iter().map(m).filter(f).reduce(l_r),
    }
}

fn orx(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => par(input.into_con_iter()).map(m).filter(f).reduce(h_r),
        false => par(input.into_con_iter()).map(m).filter(f).reduce(l_r),
    }
}

fn rayon1(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.into_par_iter().map(m).filter(f).reduce_with(h_r),
        false => input.into_par_iter().map(m).filter(f).reduce_with(l_r),
    }
}

fn rayon2(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => Some(input.into_par_iter().map(m).filter(f).reduce(|| 0, h_r)),
        false => Some(input.into_par_iter().map(m).filter(f).reduce(|| 0, l_r)),
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

    let mut group = c.benchmark_group("reduce_mfm");

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
