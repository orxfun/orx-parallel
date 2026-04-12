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

col_mf/seq/e15_light        time:   [83.107 µs 84.802 µs 86.695 µs]
col_mf/rayon/e15_light      time:   [17.899 ms 19.857 ms 22.356 ms]
col_mf/rayon_ll/e15_light   time:   [16.769 ms 17.442 ms 18.131 ms]
col_mf/orx_ord/e15_light    time:   [2.0619 ms 2.1325 ms 2.2073 ms]
col_mf/orx_arb/e15_light    time:   [2.4765 ms 2.5710 ms 2.6723 ms]
col_mf/orx_arb_vv/e15_light time:   [1.8317 ms 1.8840 ms 1.9407 ms]

col_mf/seq/e20_light        time:   [2.9597 ms 2.9956 ms 3.0325 ms]
col_mf/rayon/e20_light      time:   [21.130 ms 22.014 ms 22.856 ms]
col_mf/rayon_ll/e20_light   time:   [22.395 ms 23.000 ms 23.650 ms]
col_mf/orx_ord/e20_light    time:   [6.6131 ms 6.6927 ms 6.7757 ms]
col_mf/orx_arb/e20_light    time:   [3.8310 ms 3.8925 ms 3.9570 ms]
col_mf/orx_arb_vv/e20_light time:   [2.9780 ms 3.0128 ms 3.0486 ms]

col_mf/seq/e15_heavy        time:   [954.85 µs 963.98 µs 974.22 µs]
col_mf/rayon/e15_heavy      time:   [13.383 ms 13.634 ms 13.893 ms]
col_mf/rayon_ll/e15_heavy   time:   [14.152 ms 14.520 ms 14.902 ms]
col_mf/orx_ord/e15_heavy    time:   [2.1905 ms 2.2173 ms 2.2461 ms]
col_mf/orx_arb/e15_heavy    time:   [2.3560 ms 2.3923 ms 2.4302 ms]
col_mf/orx_arb_vv/e15_heavy time:   [2.1204 ms 2.1399 ms 2.1606 ms]

col_mf/seq/e20_heavy        time:   [30.762 ms 30.957 ms 31.153 ms]
col_mf/rayon/e20_heavy      time:   [12.065 ms 12.577 ms 13.110 ms]
col_mf/rayon_ll/e20_heavy   time:   [17.803 ms 19.211 ms 20.609 ms]
col_mf/orx_ord/e20_heavy    time:   [9.0309 ms 9.2846 ms 9.5532 ms]
col_mf/orx_arb/e20_heavy    time:   [6.8366 ms 7.0178 ms 7.2117 ms]
col_mf/orx_arb_vv/e20_heavy time:   [6.4342 ms 6.6750 ms 6.9460 ms]

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

fn orx<C: ParCollectInto<u64>>(input: &[u64], h: bool, order: IterationOrder) -> C {
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
