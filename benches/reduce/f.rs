/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used

reduce_id/seq/e15_light         time:   [4.4376 µs 4.4696 µs 4.5021 µs]
reduce_id/rayon1/e15_light      time:   [6.4945 ms 7.0677 ms 7.5915 ms]
reduce_id/rayon2/e15_light      time:   [8.6629 ms 8.7676 ms 8.8739 ms]
reduce_id/orx/e15_light         time:   [1.1026 ms 1.1162 ms 1.1299 ms]

reduce_id/seq/e20_light         time:   [222.23 µs 225.24 µs 228.35 µs]
reduce_id/rayon1/e20_light      time:   [15.130 ms 15.478 ms 15.835 ms]
reduce_id/rayon2/e20_light      time:   [15.543 ms 15.877 ms 16.216 ms]
reduce_id/orx/e20_light         time:   [1.6750 ms 1.6879 ms 1.7020 ms]

reduce_id/seq/e15_heavy         time:   [1.3085 ms 1.3181 ms 1.3288 ms]
reduce_id/rayon1/e15_heavy      time:   [9.3259 ms 9.4605 ms 9.5972 ms]
reduce_id/rayon2/e15_heavy      time:   [9.2896 ms 9.4116 ms 9.5351 ms]
reduce_id/orx/e15_heavy         time:   [1.8987 ms 1.9177 ms 1.9400 ms]

reduce_id/seq/e20_heavy         time:   [41.711 ms 41.975 ms 42.246 ms]
reduce_id/rayon1/e20_heavy      time:   [6.5043 ms 6.9336 ms 7.3821 ms]
reduce_id/rayon2/e20_heavy      time:   [7.4400 ms 7.9055 ms 8.3825 ms]
reduce_id/orx/e20_heavy         time:   [6.1684 ms 6.2234 ms 6.2801 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::{ConcurrentIter, IntoConcurrentIter};
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

fn f(a: u64) -> bool {
    !(a + 7).is_multiple_of(11)
}

fn seq(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.iter().copied().filter(f).reduce(h_r),
        false => input.iter().copied().filter(f).reduce(l_r),
    }
}

fn orx(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => par(input.into_con_iter().copied()).filter(f).reduce(h_r),
        false => par(input.into_con_iter().copied()).filter(f).reduce(l_r),
    }
}

fn rayon1(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.into_par_iter().copied().filter(f).reduce_with(h_r),
        false => input.into_par_iter().copied().filter(f).reduce_with(l_r),
    }
}

fn rayon2(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => Some(input.into_par_iter().copied().filter(f).reduce(|| 0, h_r)),
        false => Some(input.into_par_iter().copied().filter(f).reduce(|| 0, l_r)),
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

    let mut group = c.benchmark_group("reduce_id");

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
