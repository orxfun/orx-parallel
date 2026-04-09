/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used
* _ord means results are collected in order consistent to input; _arb means order might be arbitrary

col_mfmf/seq/e15_light      time:   [115.11 µs 117.39 µs 120.03 µs]
col_mfmf/rayon/e15_light    time:   [18.764 ms 20.453 ms 22.636 ms]
col_mfmf/orx_ord/e15_light  time:   [2.9880 ms 3.1119 ms 3.2395 ms]

col_mfmf/seq/e20_light      time:   [3.9800 ms 4.0413 ms 4.1042 ms]
col_mfmf/rayon/e20_light    time:   [32.316 ms 35.726 ms 39.692 ms]
col_mfmf/orx_ord/e20_light  time:   [9.4034 ms 11.075 ms 13.259 ms]

col_mfmf/seq/e15_heavy      time:   [1.5410 ms 1.5594 ms 1.5790 ms]
col_mfmf/rayon/e15_heavy    time:   [17.344 ms 19.394 ms 22.305 ms]
col_mfmf/orx_ord/e15_heavy  time:   [2.6420 ms 2.6847 ms 2.7292 ms]

col_mfmf/seq/e20_heavy      time:   [47.811 ms 48.294 ms 48.792 ms]
col_mfmf/rayon/e20_heavy    time:   [22.756 ms 26.370 ms 30.848 ms]
col_mfmf/orx_ord/e20_heavy  time:   [13.971 ms 15.924 ms 18.270 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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

fn seq(input: &[u64], h: bool) -> Vec<u64> {
    match h {
        true => input.iter().map(m).filter(f).map(h_m2).filter(f2).collect(),
        false => input.iter().map(m).filter(f).map(l_m2).filter(f2).collect(),
    }
}

fn orx(input: &[u64], h: bool, order: IterationOrder) -> Vec<u64> {
    match h {
        true => input
            .into_par()
            .iteration_order(order)
            .map(m)
            .filter(f)
            .map(h_m2)
            .filter(f2)
            .collect(),
        false => input
            .into_par()
            .iteration_order(order)
            .map(m)
            .filter(f)
            .map(l_m2)
            .filter(f2)
            .collect(),
    }
}

fn rayon(input: &[u64], h: bool) -> Vec<u64> {
    match h {
        true => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(h_m2)
            .filter(f2)
            .collect(),
        false => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(l_m2)
            .filter(f2)
            .collect(),
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

    let mut group = c.benchmark_group("col_mfmf");

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
        // let mut expected_sorted = expected.clone();
        // expected_sorted.sort();

        group.bench_with_input(BenchmarkId::new("seq", &name), &name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.heavy_compute));
            b.iter(|| seq(&input, t.heavy_compute))
        });

        group.bench_with_input(BenchmarkId::new("rayon", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon(&input, t.heavy_compute));
            b.iter(|| rayon(&input, t.heavy_compute))
        });

        group.bench_with_input(BenchmarkId::new("orx_ord", &name), &name, |b, _| {
            assert_eq!(
                &expected,
                &orx(&input, t.heavy_compute, IterationOrder::Ordered)
            );
            b.iter(|| orx(&input, t.heavy_compute, IterationOrder::Ordered))
        });

        // group.bench_with_input(BenchmarkId::new("orx_arb", &name), &name, |b, _| {
        //     let mut result = orx(&input, t.heavy_compute, IterationOrder::Arbitrary);
        //     result.sort();
        //     assert_eq!(&expected_sorted, &result);
        //     b.iter(|| orx(&input, t.heavy_compute, IterationOrder::Arbitrary))
        // });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
