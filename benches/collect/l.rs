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

col_l/seq/e15_light     time:   [322.55 µs 329.72 µs 337.38 µs]
col_l/rayon/e15_light   time:   [18.230 ms 20.710 ms 23.896 ms]
col_l/orx_ord/e15_light time:   [4.0082 ms 4.1603 ms 4.3195 ms]

col_l/seq/e20_light     time:   [38.256 ms 38.774 ms 39.325 ms]
col_l/rayon/e20_light   time:   [67.896 ms 69.129 ms 70.403 ms]
col_l/orx_ord/e20_light time:   [62.015 ms 62.809 ms 63.694 ms]

col_l/seq/e15_heavy     time:   [5.9822 ms 6.0991 ms 6.2211 ms]
col_l/rayon/e15_heavy   time:   [17.606 ms 20.055 ms 23.041 ms]
col_l/orx_ord/e15_heavy time:   [5.3735 ms 5.6578 ms 5.9529 ms]

col_l/seq/e20_heavy     time:   [186.24 ms 188.66 ms 191.21 ms]
col_l/rayon/e20_heavy   time:   [82.371 ms 85.002 ms 87.751 ms]
col_l/orx_ord/e20_heavy time:   [90.523 ms 98.385 ms 110.13 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::LinkedList;
use std::collections::LinkedList;

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

fn h_l(a: &u64) -> impl IntoIterator<Item = u64> {
    (0..7).map(move |x| fibonacci((x + a) % FIB_UPPER_BOUND))
}

fn l_l(a: &u64) -> impl IntoIterator<Item = u64> {
    (0..7).map(move |x| 2 * x + a)
}

fn seq(input: &[u64], h: bool) -> Vec<u64> {
    match h {
        true => input.iter().flat_map(h_l).collect(),
        false => input.iter().flat_map(l_l).collect(),
    }
}

fn orx<C: ParCollectInto<u64>>(input: &[u64], h: bool, order: IterationOrder) -> C {
    match h {
        true => input
            .into_par()
            .iteration_order(order)
            .flat_map(h_l)
            .collect(),
        false => input
            .into_par()
            .iteration_order(order)
            .flat_map(l_l)
            .collect(),
    }
}

fn rayon(input: &[u64], h: bool) -> Vec<u64> {
    match h {
        true => input.into_par_iter().flat_map_iter(h_l).collect(),
        false => input.into_par_iter().flat_map_iter(l_l).collect(),
    }
}

fn rayon_ll(input: &[u64]) -> LinkedList<Vec<u64>> {
    match h {
        true => input.into_par_iter().flat_map_iter(h_l).collect_vec_list(),
        false => input.into_par_iter().flat_map_iter(l_l).collect_vec_list(),
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

    let mut group = c.benchmark_group("col_l");

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
        let mut expected_sorted = expected.clone();
        expected_sorted.sort();

        group.bench_with_input(BenchmarkId::new("seq", &name), &name, |b, _| {
            assert_eq!(&expected, &seq(&input));
            b.iter(|| seq(&input))
        });

        group.bench_with_input(BenchmarkId::new("rayon", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon(&input));
            b.iter(|| rayon(&input))
        });

        group.bench_with_input(BenchmarkId::new("rayon_ll", &name), &name, |b, _| {
            let mut result: Vec<u64> = rayon_ll(&input)
                .into_iter()
                .flat_map(|x| Vec::from(x).into_iter())
                .collect();
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| rayon_ll(&input))
        });

        group.bench_with_input(BenchmarkId::new("orx_ord", &name), &name, |b, _| {
            assert_eq!(&expected, &orx::<Vec<u64>>(&input, IterationOrder::Ordered));
            b.iter(|| orx::<Vec<u64>>(&input, IterationOrder::Ordered))
        });

        group.bench_with_input(BenchmarkId::new("orx_arb", &name), &name, |b, _| {
            let mut result: Vec<u64> = orx(&input, IterationOrder::Arbitrary);
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| orx::<Vec<u64>>(&input, IterationOrder::Arbitrary))
        });

        group.bench_with_input(BenchmarkId::new("orx_arb_vv", &name), &name, |b, _| {
            let mut result: Vec<u64> = orx::<Vec<Vec<_>>>(&input, IterationOrder::Arbitrary)
                .into_iter()
                .flatten()
                .collect();
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| orx::<Vec<Vec<_>>>(&input, IterationOrder::Arbitrary))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
