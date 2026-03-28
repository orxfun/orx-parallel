/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used

reduce_mfm/seq/e15_light        time:   [33.483 µs 33.782 µs 34.109 µs]
reduce_mfm/rayon1/e15_light     time:   [11.854 ms 13.804 ms 16.477 ms]
reduce_mfm/rayon2/e15_light     time:   [13.994 ms 15.436 ms 17.134 ms]
reduce_mfm/orx/e15_light        time:   [2.2035 ms 2.5088 ms 2.9190 ms]

reduce_mfm/seq/e20_light        time:   [1.2638 ms 1.3150 ms 1.3837 ms]
reduce_mfm/rayon1/e20_light     time:   [22.970 ms 26.492 ms 31.596 ms]
reduce_mfm/rayon2/e20_light     time:   [27.670 ms 32.707 ms 38.912 ms]
reduce_mfm/orx/e20_light        time:   [3.5112 ms 3.5976 ms 3.6888 ms]

reduce_mfm/seq/e15_heavy        time:   [1.9552 ms 2.0777 ms 2.2195 ms]
reduce_mfm/rayon1/e15_heavy     time:   [18.669 ms 20.875 ms 23.541 ms]
reduce_mfm/rayon2/e15_heavy     time:   [13.667 ms 15.465 ms 18.235 ms]
reduce_mfm/orx/e15_heavy        time:   [2.8618 ms 3.4951 ms 4.4819 ms]

reduce_mfm/seq/e20_heavy        time:   [50.887 ms 51.456 ms 52.058 ms]
reduce_mfm/rayon1/e20_heavy     time:   [16.757 ms 22.586 ms 29.826 ms]
reduce_mfm/rayon2/e20_heavy     time:   [11.180 ms 12.028 ms 12.916 ms]
reduce_mfm/orx/e20_heavy        time:   [9.8016 ms 11.058 ms 12.773 ms]

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
