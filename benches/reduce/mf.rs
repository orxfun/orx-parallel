/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used

reduce_mf/seq/e15_light     time:   [28.649 µs 28.893 µs 29.153 µs]
reduce_mf/rayon1/e15_light  time:   [8.5049 ms 8.9180 ms 9.2979 ms]
reduce_mf/rayon2/e15_light  time:   [9.0417 ms 9.6792 ms 10.271 ms]
reduce_mf/orx/e15_light     time:   [1.4126 ms 1.5080 ms 1.6064 ms]

reduce_mf/seq/e20_light     time:   [1.0508 ms 1.0671 ms 1.0856 ms]
reduce_mf/rayon1/e20_light  time:   [17.332 ms 17.716 ms 18.110 ms]
reduce_mf/rayon2/e20_light  time:   [18.626 ms 19.347 ms 20.062 ms]
reduce_mf/orx/e20_light     time:   [2.3823 ms 2.4739 ms 2.5784 ms]

reduce_mf/seq/e15_heavy     time:   [1.4456 ms 1.4654 ms 1.4874 ms]
reduce_mf/rayon1/e15_heavy  time:   [8.8519 ms 9.5472 ms 10.200 ms]
reduce_mf/rayon2/e15_heavy  time:   [11.701 ms 12.145 ms 12.598 ms]
reduce_mf/orx/e15_heavy     time:   [2.1108 ms 2.1305 ms 2.1511 ms]

reduce_mf/seq/e20_heavy     time:   [49.205 ms 50.786 ms 52.572 ms]
reduce_mf/rayon1/e20_heavy  time:   [8.0877 ms 8.6066 ms 9.1513 ms]
reduce_mf/rayon2/e20_heavy  time:   [9.3101 ms 10.033 ms 10.775 ms]
reduce_mf/orx/e20_heavy     time:   [7.6426 ms 7.8071 ms 7.9760 ms]

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

fn seq(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.iter().map(m).filter(f).reduce(h_r),
        false => input.iter().map(m).filter(f).reduce(l_r),
    }
}

fn orx(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input.into_par().map(m).filter(f).reduce(h_r),
        false => input.into_par().map(m).filter(f).reduce(l_r),
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

    let mut group = c.benchmark_group("reduce_mf");

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
