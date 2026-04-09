/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used

reduce_mfm/seq/e15_light    time:   [46.793 µs 47.343 µs 47.885 µs]
reduce_mfm/rayon1/e15_light time:   [7.5122 ms 7.9451 ms 8.3244 ms]
reduce_mfm/rayon2/e15_light time:   [9.1598 ms 9.2558 ms 9.3536 ms]
reduce_mfm/orx/e15_light    time:   [1.2662 ms 1.2887 ms 1.3183 ms]

reduce_mfm/seq/e20_light    time:   [2.3262 ms 2.3455 ms 2.3663 ms]
reduce_mfm/rayon1/e20_light time:   [16.413 ms 16.736 ms 17.069 ms]
reduce_mfm/rayon2/e20_light time:   [16.353 ms 16.869 ms 17.387 ms]
reduce_mfm/orx/e20_light    time:   [2.1666 ms 2.1964 ms 2.2374 ms]

reduce_mfm/seq/e15_heavy    time:   [2.2362 ms 2.2499 ms 2.2647 ms]
reduce_mfm/rayon1/e15_heavy time:   [10.136 ms 10.261 ms 10.388 ms]
reduce_mfm/rayon2/e15_heavy time:   [10.403 ms 10.609 ms 10.819 ms]
reduce_mfm/orx/e15_heavy    time:   [2.1472 ms 2.1612 ms 2.1755 ms]

reduce_mfm/seq/e20_heavy    time:   [70.467 ms 70.860 ms 71.267 ms]
reduce_mfm/rayon1/e20_heavy time:   [10.125 ms 10.531 ms 10.939 ms]
reduce_mfm/rayon2/e20_heavy time:   [10.316 ms 10.746 ms 11.199 ms]
reduce_mfm/orx/e20_heavy    time:   [10.295 ms 10.376 ms 10.458 ms]

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

fn seq(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.iter().map(m).filter(f).map(h_m2).reduce(h_r),
        false => input.iter().map(m).filter(f).map(l_m2).reduce(l_r),
    }
}

fn orx(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.into_par().map(m).filter(f).map(h_m2).reduce(h_r),
        false => input.into_par().map(m).filter(f).map(l_m2).reduce(l_r),
    }
}

fn rayon1(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(h_m2)
            .reduce_with(h_r),
        false => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(l_m2)
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
                .reduce(|| 0, h_r),
        ),
        false => Some(
            input
                .into_par_iter()
                .map(m)
                .filter(f)
                .map(l_m2)
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
