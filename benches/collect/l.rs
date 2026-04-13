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
  * Note that _ll and _vv 2-dim jagged results in rayon and orx, respectively

col_l/seq/e15_light         time:   [254.31 µs 262.77 µs 271.04 µs]
col_l/rayon/e15_light       time:   [11.287 ms 11.470 ms 11.656 ms]
col_l/rayon_ll/e15_light    time:   [11.289 ms 11.446 ms 11.606 ms]
col_l/orx_ord/e15_light     time:   [2.1541 ms 2.1855 ms 2.2247 ms]
col_l/orx_arb/e15_light     time:   [2.1304 ms 2.1557 ms 2.1831 ms]
col_l/orx_arb_vv/e15_light  time:   [1.9153 ms 1.9294 ms 1.9448 ms]

col_l/seq/e20_light         time:   [35.633 ms 35.892 ms 36.161 ms]
col_l/rayon/e20_light       time:   [50.719 ms 51.396 ms 52.096 ms]
col_l/rayon_ll/e20_light    time:   [17.909 ms 18.282 ms 18.664 ms]
col_l/orx_ord/e20_light     time:   [36.063 ms 36.309 ms 36.559 ms]
col_l/orx_arb/e20_light     time:   [33.052 ms 33.308 ms 33.567 ms]
col_l/orx_arb_vv/e20_light  time:   [3.6852 ms 3.7484 ms 3.8142 ms]

col_l/seq/e15_heavy         time:   [4.0291 ms 4.0556 ms 4.0817 ms]
col_l/rayon/e15_heavy       time:   [12.105 ms 12.317 ms 12.532 ms]
col_l/rayon_ll/e15_heavy    time:   [12.642 ms 12.877 ms 13.107 ms]
col_l/orx_ord/e15_heavy     time:   [2.8103 ms 2.8659 ms 2.9333 ms]
col_l/orx_arb/e15_heavy     time:   [2.7724 ms 2.7950 ms 2.8181 ms]
col_l/orx_arb_vv/e15_heavy  time:   [2.6090 ms 2.6334 ms 2.6591 ms]

col_l/seq/e20_heavy         time:   [148.89 ms 149.97 ms 151.11 ms]
col_l/rayon/e20_heavy       time:   [59.823 ms 61.186 ms 62.615 ms]
col_l/rayon_ll/e20_heavy    time:   [27.061 ms 27.860 ms 28.660 ms]
col_l/orx_ord/e20_heavy     time:   [50.841 ms 51.711 ms 52.683 ms]
col_l/orx_arb/e20_heavy     time:   [56.696 ms 58.252 ms 59.845 ms]
col_l/orx_arb_vv/e20_heavy  time:   [16.114 ms 16.461 ms 16.817 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
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

fn rayon_ll(input: &[u64], h: bool) -> LinkedList<Vec<u64>> {
    match h {
        true => input.into_par_iter().flat_map_iter(h_l).collect_vec_list(),
        false => input.into_par_iter().flat_map_iter(l_l).collect_vec_list(),
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

    let mut group = c.benchmark_group("col_l");

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
