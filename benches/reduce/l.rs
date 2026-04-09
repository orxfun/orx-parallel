/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used

reduce_l/seq/e15_light      time:   [20.289 ms 20.503 ms 20.717 ms]
reduce_l/rayon1/e15_light   time:   [26.610 ms 27.316 ms 28.066 ms]
reduce_l/rayon2/e15_light   time:   [27.028 ms 27.764 ms 28.520 ms]
reduce_l/orx/e15_light      time:   [10.564 ms 10.678 ms 10.800 ms]

reduce_l/seq/e20_light      time:   [640.82 ms 649.27 ms 658.60 ms]
reduce_l/rayon1/e20_light   time:   [53.800 ms 55.471 ms 57.235 ms]
reduce_l/rayon2/e20_light   time:   [54.587 ms 56.077 ms 57.589 ms]
reduce_l/orx/e20_light      time:   [39.367 ms 40.115 ms 40.901 ms]

reduce_l/seq/e15_heavy      time:   [57.803 ms 58.304 ms 58.841 ms]
reduce_l/rayon1/e15_heavy   time:   [26.431 ms 27.709 ms 29.032 ms]
reduce_l/rayon2/e15_heavy   time:   [25.870 ms 26.534 ms 27.215 ms]
reduce_l/orx/e15_heavy      time:   [11.205 ms 11.326 ms 11.450 ms]

reduce_l/seq/e20_heavy      time:   [1.8874 s 1.9025 s 1.9188 s]
reduce_l/rayon1/e20_heavy   time:   [82.763 ms 84.442 ms 86.147 ms]
reduce_l/rayon2/e20_heavy   time:   [97.017 ms 100.36 ms 104.11 ms]
reduce_l/orx/e20_heavy      time:   [76.860 ms 77.978 ms 79.163 ms]


reduce_l/seq/e15_light      time:   [3.2407 ms 3.2770 ms 3.3170 ms]
reduce_l/rayon1/e15_light   time:   [10.063 ms 10.239 ms 10.414 ms]
reduce_l/rayon2/e15_light   time:   [11.104 ms 11.349 ms 11.589 ms]
reduce_l/orx/e15_light      time:   [2.8054 ms 2.8736 ms 2.9444 ms]

reduce_l/seq/e20_light      time:   [119.21 ms 122.20 ms 125.72 ms]
reduce_l/rayon1/e20_light   time:   [19.271 ms 19.931 ms 20.606 ms]
reduce_l/rayon2/e20_light   time:   [24.032 ms 25.386 ms 26.871 ms]
reduce_l/orx/e20_light      time:   [18.195 ms 18.805 ms 19.523 ms]

reduce_l/seq/e15_heavy      time:   [10.685 ms 10.773 ms 10.866 ms]
reduce_l/rayon1/e15_heavy   time:   [5.6738 ms 6.3265 ms 7.0368 ms]
reduce_l/rayon2/e15_heavy   time:   [4.2011 ms 4.6651 ms 5.1503 ms]
reduce_l/orx/e15_heavy      time:   [3.5605 ms 3.6242 ms 3.6911 ms]

reduce_l/seq/e20_heavy      time:   [326.97 ms 331.60 ms 336.92 ms]
reduce_l/rayon1/e20_heavy   time:   [34.614 ms 35.333 ms 36.052 ms]
reduce_l/rayon2/e20_heavy   time:   [44.882 ms 46.087 ms 47.285 ms]
reduce_l/orx/e20_heavy      time:   [42.330 ms 43.862 ms 45.625 ms]

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

fn l_l(a: &u64) -> impl IntoIterator<Item = u64> {
    (0..7).map(move |x| 2 * x + a)
}

fn h_l(a: &u64) -> impl IntoIterator<Item = u64> {
    (0..7).map(move |x| fibonacci((x + a) % FIB_UPPER_BOUND))
}

fn seq(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.iter().flat_map(l_l).reduce(h_r),
        false => input.iter().flat_map(h_l).reduce(l_r),
    }
}

fn orx(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.into_par().flat_map(l_l).reduce(h_r),
        false => input.into_par().flat_map(h_l).reduce(l_r),
    }
}

fn rayon1(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.into_par_iter().flat_map_iter(l_l).reduce_with(h_r),
        false => input.into_par_iter().flat_map_iter(h_l).reduce_with(l_r),
    }
}

fn rayon2(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => Some(input.into_par_iter().flat_map_iter(l_l).reduce(|| 0, h_r)),
        false => Some(input.into_par_iter().flat_map_iter(h_l).reduce(|| 0, l_r)),
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

    let mut group = c.benchmark_group("reduce_l");

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
