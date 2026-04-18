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

col_m/seq/e15_light         time:   [18.493 µs 18.626 µs 18.770 µs]
col_m/rayon/e15_light       time:   [12.343 ms 12.662 ms 12.993 ms]
col_m/rayon_ll/e15_light    time:   [12.981 ms 13.350 ms 13.707 ms]
col_m/orx_ord/e15_light     time:   [1.7462 ms 1.8022 ms 1.8698 ms]
col_m/orx_arb/e15_light     time:   [1.7914 ms 1.8621 ms 1.9365 ms]
col_m/orx_arb_vv/e15_light  time:   [1.7587 ms 1.8129 ms 1.8782 ms]

col_m/seq/e20_light         time:   [841.69 µs 852.07 µs 863.24 µs]
col_m/rayon/e20_light       time:   [23.285 ms 24.143 ms 25.028 ms]
col_m/rayon_ll/e20_light    time:   [19.368 ms 20.015 ms 20.677 ms]
col_m/orx_ord/e20_light     time:   [3.9208 ms 4.0096 ms 4.1061 ms]
col_m/orx_arb/e20_light     time:   [4.6995 ms 4.8415 ms 4.9916 ms]
col_m/orx_arb_vv/e20_light  time:   [3.0450 ms 3.1517 ms 3.2628 ms]

col_m/seq/e15_heavy         time:   [990.17 µs 1.0076 ms 1.0268 ms]
col_m/rayon/e15_heavy       time:   [14.316 ms 15.943 ms 18.413 ms]
col_m/rayon_ll/e15_heavy    time:   [14.463 ms 14.827 ms 15.196 ms]
col_m/orx_ord/e15_heavy     time:   [2.7481 ms 2.7919 ms 2.8382 ms]
col_m/orx_arb/e15_heavy     time:   [2.5376 ms 2.5685 ms 2.6009 ms]
col_m/orx_arb_vv/e15_heavy  time:   [2.4474 ms 2.4789 ms 2.5130 ms]

col_m/seq/e20_heavy         time:   [33.216 ms 33.627 ms 34.055 ms]
col_m/rayon/e20_heavy       time:   [16.425 ms 17.658 ms 19.029 ms]
col_m/rayon_ll/e20_heavy    time:   [19.459 ms 21.203 ms 23.007 ms]
col_m/orx_ord/e20_heavy     time:   [8.1529 ms 8.4001 ms 8.6562 ms]
col_m/orx_arb/e20_heavy     time:   [7.5584 ms 7.7608 ms 7.9701 ms]
col_m/orx_arb_vv/e20_heavy  time:   [7.4511 ms 7.7397 ms 8.0443 ms]

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

fn orx<C: ParCollectInto<u64>>(input: &[u64], h: bool, order: IterationOrder) -> C {
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
