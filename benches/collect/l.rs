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
  * Note that _vv and _ll are comparable jagged results in rayon and orx

col_l/seq/e15_light         time:   [310.58 µs 316.34 µs 322.19 µs]
col_l/rayon/e15_light       time:   [17.621 ms 19.447 ms 22.047 ms]
col_l/rayon_ll/e15_light    time:   [18.666 ms 21.728 ms 25.493 ms]
col_l/orx_ord/e15_light     time:   [4.3176 ms 4.8553 ms 5.4523 ms]
col_l/orx_arb/e15_light     time:   [3.1877 ms 3.4021 ms 3.6311 ms]
col_l/orx_arb_vv/e15_light  time:   [2.6187 ms 2.7108 ms 2.8129 ms]

col_l/seq/e20_light         time:   [50.029 ms 50.897 ms 51.766 ms]
col_l/rayon/e20_light       time:   [80.782 ms 83.652 ms 87.101 ms]
col_l/rayon_ll/e20_light    time:   [28.918 ms 30.794 ms 32.890 ms]
col_l/orx_ord/e20_light     time:   [109.45 ms 115.00 ms 120.96 ms]
col_l/orx_arb/e20_light     time:   [55.961 ms 59.554 ms 63.925 ms]
col_l/orx_arb_vv/e20_light  time:   [5.1614 ms 5.3643 ms 5.5745 ms]

col_l/seq/e15_heavy         time:   [5.5437 ms 5.6550 ms 5.7695 ms]
col_l/rayon/e15_heavy       time:   [19.012 ms 20.622 ms 22.442 ms]
col_l/rayon_ll/e15_heavy    time:   [20.195 ms 21.398 ms 22.673 ms]
col_l/orx_ord/e15_heavy     time:   [5.7195 ms 6.3956 ms 7.3053 ms]
col_l/orx_arb/e15_heavy     time:   [5.1585 ms 6.0944 ms 7.3499 ms]
col_l/orx_arb_vv/e15_heavy  time:   [3.6555 ms 3.7629 ms 3.8724 ms]

col_l/seq/e20_heavy         time:   [207.11 ms 211.85 ms 217.31 ms]
col_l/rayon/e20_heavy       time:   [103.33 ms 110.11 ms 119.74 ms]
col_l/rayon_ll/e20_heavy    time:   [35.907 ms 38.495 ms 41.586 ms]
col_l/orx_ord/e20_heavy     time:   [100.94 ms 104.27 ms 108.05 ms]
col_l/orx_arb/e20_heavy     time:   [70.276 ms 75.238 ms 83.013 ms]
col_l/orx_arb_vv/e20_heavy  time:   [18.355 ms 19.550 ms 20.954 ms]

// TODO: great room for improvement in ordering



-> merge_ord_into1
col_l/orx_ord/e20_light time:   [66.089 ms 66.850 ms 67.638 ms]
col_l/orx_arb/e20_light time:   [51.682 ms 52.492 ms 53.323 ms]

-> merge_ord_into2
col_l/orx_ord/e20_light time:   [251.86 ms 262.88 ms 275.03 ms]
col_l/orx_arb/e20_light time:   [45.270 ms 45.891 ms 46.546 ms]

-> merge_ord_into3
col_l/orx_ord/e20_light time:   [278.18 ms 282.93 ms 287.73 ms]
col_l/orx_arb/e20_light time:   [42.429 ms 42.814 ms 43.222 ms]

-> merge_ord_into4
col_l/orx_ord/e20_light time:   [495.96 ms 503.77 ms 511.77 ms]
col_l/orx_arb/e20_light time:   [46.768 ms 47.367 ms 48.001 ms]

-> merge_ord_into5
col_l/orx_ord/e20_light time:   [381.38 ms 387.99 ms 394.68 ms]
col_l/orx_arb/e20_light time:   [50.448 ms 51.741 ms 53.124 ms]

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
