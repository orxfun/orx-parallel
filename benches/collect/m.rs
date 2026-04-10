/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used
* order:
  * _ord means results are collected in order leading to same result as sequential
  * _arb means results are collected in arbitrary order
* container:
  * _vec means, results are collected into a Vec
  * _vv means, results are collected into a Vec<Vec<_>>
  * _ll means, results are collected into a LinkedList<Vec<_>>

col_m/seq/e15_light     time:   [18.924 µs 19.104 µs 19.296 µs]
col_m/rayon/e15_light   time:   [12.222 ms 13.502 ms 15.522 ms]
col_m/orx_ord/e15_light time:   [1.8075 ms 1.8505 ms 1.8980 ms]
col_m/orx_arb/e15_light time:   [4.8928 ms 5.0619 ms 5.2402 ms]

col_m/seq/e20_light     time:   [808.84 µs 819.39 µs 830.30 µs]
col_m/rayon/e20_light   time:   [29.515 ms 32.127 ms 35.009 ms]
col_m/orx_ord/e20_light time:   [7.8373 ms 8.0218 ms 8.2136 ms]
col_m/orx_arb/e20_light time:   [135.55 ms 151.97 ms 173.12 ms]

col_m/seq/e15_heavy     time:   [1.0235 ms 1.0460 ms 1.0701 ms]
col_m/rayon/e15_heavy   time:   [15.338 ms 16.081 ms 16.870 ms]
col_m/orx_ord/e15_heavy time:   [3.0646 ms 3.1532 ms 3.2492 ms]
col_m/orx_arb/e15_heavy time:   [7.2354 ms 8.4175 ms 9.9316 ms]

col_m/seq/e20_heavy     time:   [33.794 ms 34.232 ms 34.690 ms]
col_m/rayon/e20_heavy   time:   [16.382 ms 19.215 ms 22.916 ms]
col_m/orx_ord/e20_heavy time:   [12.410 ms 13.334 ms 14.361 ms]
col_m/orx_arb/e20_heavy time:   [137.07 ms 150.09 ms 165.88 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::LinkedList;
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

fn l_m(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

fn h_m(x: &u64) -> u64 {
    let f = black_box(fibonacci(*x % FIB_UPPER_BOUND));
    let g = black_box(*x + f);
    match *x {
        999 => g - f,
        n => 7 * n + 1000,
    }
}

fn seq(input: &[u64], h: bool) -> Vec<u64> {
    match h {
        true => input.iter().map(h_m).collect(),
        false => input.iter().map(l_m).collect(),
    }
}

fn orx(input: &[u64], h: bool, order: IterationOrder) -> Vec<u64> {
    match h {
        true => input.into_par().iteration_order(order).map(h_m).collect(),
        false => input.into_par().iteration_order(order).map(l_m).collect(),
    }
}

fn rayon(input: &[u64], h: bool) -> Vec<u64> {
    match h {
        true => input.into_par_iter().map(h_m).collect(),
        false => input.into_par_iter().map(l_m).collect(),
    }
}

fn rayon_ll(input: &[u64], h: bool) -> LinkedList<Vec<u64>> {
    match h {
        true => input.into_par_iter().map(h_m).collect_vec_list(),
        false => input.into_par_iter().map(l_m).collect_vec_list(),
    }
}

struct Treat {
    len: usize,
    heavy: bool,
}

fn run(c: &mut Criterion) {
    let treatments = [
        Treat {
            len: 1 << 15,
            heavy: false,
        },
        Treat {
            len: 1 << 20,
            heavy: false,
        },
        Treat {
            len: 1 << 15,
            heavy: true,
        },
        Treat {
            len: 1 << 20,
            heavy: true,
        },
    ];

    let mut group = c.benchmark_group("col_m");

    for t in treatments {
        let name = format!(
            "e{}_{}",
            t.len.ilog2(),
            match t.heavy {
                true => "heavy",
                false => "light",
            },
        );
        let input = inputs(t.len);
        let expected = seq(&input, t.heavy);
        let mut expected_sorted = expected.clone();
        expected_sorted.sort();

        group.bench_with_input(BenchmarkId::new("seq", &name), &name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.heavy));
            b.iter(|| seq(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("rayon", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon(&input, t.heavy));
            b.iter(|| rayon(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("rayon_ll", &name), &name, |b, _| {
            let mut result: Vec<u64> = rayon_ll(&input, t.heavy)
                .into_iter()
                .flat_map(|x| Vec::from(x).into_iter())
                .collect();
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| rayon_ll(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("orx_ord", &name), &name, |b, _| {
            assert_eq!(
                &expected,
                &orx::<Vec<u64>>(&input, t.heavy, IterationOrder::Ordered)
            );
            b.iter(|| orx::<Vec<u64>>(&input, t.heavy, IterationOrder::Ordered))
        });

        group.bench_with_input(BenchmarkId::new("orx_arb", &name), &name, |b, _| {
            let mut result: Vec<u64> = orx(&input, t.heavy, IterationOrder::Arbitrary);
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| orx::<Vec<u64>>(&input, t.heavy, IterationOrder::Arbitrary))
        });

        group.bench_with_input(BenchmarkId::new("orx_arb_vv", &name), &name, |b, _| {
            let mut result: Vec<u64> =
                orx::<Vec<Vec<_>>>(&input, t.heavy, IterationOrder::Arbitrary)
                    .into_iter()
                    .flatten()
                    .collect();
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| orx::<Vec<Vec<_>>>(&input, t.heavy, IterationOrder::Arbitrary))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
