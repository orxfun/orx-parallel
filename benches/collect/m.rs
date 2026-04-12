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
  * Note that _vv and _ll are corresponding jagged results in rayon and orx

col_m/seq/e15_light         time:   [22.825 µs 23.202 µs 23.589 µs]
col_m/rayon/e15_light       time:   [16.141 ms 16.897 ms 17.698 ms]
col_m/rayon_ll/e15_light    time:   [14.953 ms 17.651 ms 21.332 ms]
col_m/orx_ord/e15_light     time:   [2.3073 ms 2.3842 ms 2.4637 ms]
col_m/orx_arb/e15_light     time:   [2.8894 ms 3.5439 ms 4.4601 ms]
col_m/orx_arb_vv/e15_light  time:   [2.7929 ms 3.0373 ms 3.3083 ms]

col_m/seq/e20_light         time:   [978.74 µs 990.66 µs 1.0023 ms]
col_m/rayon/e20_light       time:   [26.087 ms 27.974 ms 30.075 ms]
col_m/rayon_ll/e20_light    time:   [22.957 ms 25.167 ms 27.745 ms]
col_m/orx_ord/e20_light     time:   [11.161 ms 11.473 ms 11.786 ms]
col_m/orx_arb/e20_light     time:   [6.8536 ms 7.7292 ms 8.9571 ms]
col_m/orx_arb_vv/e20_light  time:   [3.6359 ms 3.7836 ms 3.9417 ms]

col_m/seq/e15_heavy         time:   [1.2543 ms 1.2913 ms 1.3275 ms]
col_m/rayon/e15_heavy       time:   [16.049 ms 17.155 ms 18.497 ms]
col_m/rayon_ll/e15_heavy    time:   [17.484 ms 20.037 ms 23.373 ms]
col_m/orx_ord/e15_heavy     time:   [4.1418 ms 4.3987 ms 4.6662 ms]
col_m/orx_arb/e15_heavy     time:   [3.4482 ms 3.6250 ms 3.8053 ms]
col_m/orx_arb_vv/e15_heavy  time:   [3.4609 ms 3.5844 ms 3.7261 ms]

col_m/seq/e20_heavy         time:   [42.535 ms 43.081 ms 43.614 ms]
col_m/rayon/e20_heavy       time:   [20.826 ms 23.846 ms 27.151 ms]
col_m/rayon_ll/e20_heavy    time:   [21.314 ms 24.467 ms 28.319 ms]
col_m/orx_ord/e20_heavy     time:   [11.838 ms 13.506 ms 15.831 ms]
col_m/orx_arb/e20_heavy     time:   [8.5316 ms 9.3846 ms 10.459 ms]
col_m/orx_arb_vv/e20_heavy  time:   [8.8921 ms 9.6499 ms 10.708 ms]

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
