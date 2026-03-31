/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used

reduce_id/seq/e15_light     time:   [4.7806 µs 4.8926 µs 4.9974 µs]
reduce_id/rayon1/e15_light  time:   [8.4413 ms 8.7003 ms 8.9418 ms]
reduce_id/rayon2/e15_light  time:   [9.7850 ms 9.9723 ms 10.161 ms]
reduce_id/orx/e15_light     time:   [1.2602 ms 1.2748 ms 1.2906 ms]

reduce_id/seq/e20_light     time:   [232.11 µs 234.04 µs 236.05 µs]
reduce_id/rayon1/e20_light  time:   [17.857 ms 18.285 ms 18.712 ms]
reduce_id/rayon2/e20_light  time:   [17.781 ms 18.538 ms 19.279 ms]
reduce_id/orx/e20_light     time:   [2.0632 ms 2.0930 ms 2.1245 ms]

reduce_id/seq/e15_heavy     time:   [1.4879 ms 1.5022 ms 1.5188 ms]
reduce_id/rayon1/e15_heavy  time:   [10.655 ms 10.904 ms 11.147 ms]
reduce_id/rayon2/e15_heavy  time:   [10.854 ms 11.052 ms 11.250 ms]
reduce_id/orx/e15_heavy     time:   [2.2717 ms 2.3045 ms 2.3441 ms]

reduce_id/seq/e20_heavy     time:   [49.829 ms 50.799 ms 51.830 ms]
reduce_id/rayon1/e20_heavy  time:   [13.010 ms 14.283 ms 15.763 ms]
reduce_id/rayon2/e20_heavy  time:   [15.656 ms 16.773 ms 18.057 ms]
reduce_id/orx/e20_heavy     time:   [7.8542 ms 8.1934 ms 8.6432 ms]

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

fn seq(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.iter().copied().reduce(h_r),
        false => input.iter().copied().reduce(l_r),
    }
}

fn orx(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => par(input.into_con_iter()).copied().reduce(h_r),
        false => par(input.into_con_iter()).copied().reduce(l_r),
    }
}

fn rayon1(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.into_par_iter().copied().reduce_with(h_r),
        false => input.into_par_iter().copied().reduce_with(l_r),
    }
}

fn rayon2(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => Some(input.into_par_iter().copied().reduce(|| 0, h_r)),
        false => Some(input.into_par_iter().copied().reduce(|| 0, l_r)),
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
