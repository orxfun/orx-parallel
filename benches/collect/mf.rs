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

col_mf/seq/e15_light        time:   [99.285 µs 100.95 µs 102.72 µs]
col_mf/rayon/e15_light      time:   [20.741 ms 21.963 ms 23.304 ms]
col_mf/orx_ord/e15_light    time:   [2.7294 ms 3.0338 ms 3.3875 ms]
col_mf/orx_arb/e15_light    time:   [5.3800 ms 5.6940 ms 6.0181 ms]

col_mf/seq/e20_light        time:   [3.0834 ms 3.1208 ms 3.1592 ms]
col_mf/rayon/e20_light      time:   [32.975 ms 36.437 ms 40.543 ms]
col_mf/orx_ord/e20_light    time:   [7.7600 ms 7.8839 ms 8.0112 ms]
col_mf/orx_arb/e20_light    time:   [104.03 ms 109.71 ms 116.01 ms]

col_mf/seq/e15_heavy        time:   [1.1029 ms 1.1233 ms 1.1459 ms]
col_mf/rayon/e15_heavy      time:   [20.927 ms 22.352 ms 23.923 ms]
col_mf/orx_ord/e15_heavy    time:   [2.8730 ms 2.9539 ms 3.0389 ms]
col_mf/orx_arb/e15_heavy    time:   [4.8328 ms 5.0729 ms 5.3300 ms]

col_mf/seq/e20_heavy        time:   [32.445 ms 32.816 ms 33.193 ms]
col_mf/rayon/e20_heavy      time:   [18.508 ms 19.692 ms 20.939 ms]
col_mf/orx_ord/e20_heavy    time:   [10.370 ms 12.144 ms 14.507 ms]
col_mf/orx_arb/e20_heavy    time:   [113.53 ms 125.99 ms 141.06 ms]

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

fn f(a: &u64) -> bool {
    !(a + 7).is_multiple_of(11)
}

fn seq(input: &[u64], h: bool) -> Vec<u64> {
    match h {
        true => input.iter().map(h_m).filter(f).collect(),
        false => input.iter().map(l_m).filter(f).collect(),
    }
}

fn orx(input: &[u64], h: bool, order: IterationOrder) -> Vec<u64> {
    match h {
        true => input
            .into_par()
            .iteration_order(order)
            .map(h_m)
            .filter(f)
            .collect(),
        false => input
            .into_par()
            .iteration_order(order)
            .map(l_m)
            .filter(f)
            .collect(),
    }
}

fn rayon(input: &[u64], h: bool) -> Vec<u64> {
    match h {
        true => input.into_par_iter().map(h_m).filter(f).collect(),
        false => input.into_par_iter().map(l_m).filter(f).collect(),
    }
}

fn rayon_ll(input: &[u64], h: bool) -> LinkedList<Vec<u64>> {
    match h {
        true => input.into_par_iter().map(h_m).filter(f).collect_vec_list(),
        false => input.into_par_iter().map(l_m).filter(f).collect_vec_list(),
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

    let mut group = c.benchmark_group("col_mf");

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
