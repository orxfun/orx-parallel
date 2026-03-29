/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used

reduce_mfmf/seq/e15_light       time:   [436.72 µs 445.66 µs 454.85 µs]
reduce_mfmf/rayon1/e15_light    time:   [29.689 ms 30.899 ms 32.159 ms]
reduce_mfmf/rayon2/e15_light    time:   [26.592 ms 27.356 ms 28.136 ms]
reduce_mfmf/orx/e15_light       time:   [8.9992 ms 9.1352 ms 9.2738 ms]

reduce_mfmf/seq/e20_light       time:   [17.886 ms 18.446 ms 19.057 ms]
reduce_mfmf/rayon1/e20_light    time:   [55.202 ms 58.669 ms 62.224 ms]
reduce_mfmf/rayon2/e20_light    time:   [56.507 ms 59.712 ms 63.064 ms]
reduce_mfmf/orx/e20_light       time:   [10.364 ms 10.443 ms 10.524 ms]

reduce_mfmf/seq/e15_heavy       time:   [13.957 ms 14.042 ms 14.135 ms]
reduce_mfmf/rayon1/e15_heavy    time:   [25.345 ms 26.036 ms 26.741 ms]
reduce_mfmf/rayon2/e15_heavy    time:   [26.091 ms 26.808 ms 27.542 ms]
reduce_mfmf/orx/e15_heavy       time:   [9.9915 ms 10.087 ms 10.192 ms]

reduce_mfmf/seq/e20_heavy       time:   [432.92 ms 439.28 ms 446.37 ms]
reduce_mfmf/rayon1/e20_heavy    time:   [39.678 ms 40.471 ms 41.272 ms]
reduce_mfmf/rayon2/e20_heavy    time:   [38.366 ms 39.181 ms 39.991 ms]
reduce_mfmf/orx/e20_heavy       time:   [24.227 ms 24.533 ms 24.854 ms]

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

fn h_m2(x: u64) -> u64 {
    match x {
        999 => 999,
        n => fibonacci(n % FIB_UPPER_BOUND) + 1000,
    }
}

fn l_m2(x: u64) -> u64 {
    match x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

fn f2(a: &u64) -> bool {
    !(2 * a + 11).is_multiple_of(7)
}

fn seq(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input
            .iter()
            .map(m)
            .filter(f)
            .map(h_m2)
            .filter(f2)
            .reduce(h_r),
        false => input
            .iter()
            .map(m)
            .filter(f)
            .map(l_m2)
            .filter(f2)
            .reduce(l_r),
    }
}

fn orx(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => par(input.into_con_iter())
            .map(m)
            .filter(f)
            .map(h_m2)
            .filter(f2)
            .reduce(h_r),
        false => par(input.into_con_iter())
            .map(m)
            .filter(f)
            .map(l_m2)
            .filter(f2)
            .reduce(l_r),
    }
}

fn rayon1(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(h_m2)
            .filter(f2)
            .reduce_with(h_r),
        false => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(l_m2)
            .filter(f2)
            .reduce_with(l_r),
    }
}

fn rayon2(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => Some(
            input
                .into_par_iter()
                .map(m)
                .filter(f)
                .map(h_m2)
                .filter(f2)
                .reduce(|| 0, h_r),
        ),
        false => Some(
            input
                .into_par_iter()
                .map(m)
                .filter(f)
                .map(l_m2)
                .filter(f2)
                .reduce(|| 0, l_r),
        ),
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

    let mut group = c.benchmark_group("reduce_mfmf");

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
