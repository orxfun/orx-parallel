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

col_mf/seq/e15_light        time:   [77.815 µs 79.077 µs 80.463 µs]
col_mf/rayon/e15_light      time:   [14.826 ms 15.148 ms 15.475 ms]
col_mf/rayon_ll/e15_light   time:   [14.269 ms 14.642 ms 15.026 ms]
col_mf/orx_ord/e15_light    time:   [2.0284 ms 2.0699 ms 2.1134 ms]
col_mf/orx_arb/e15_light    time:   [1.8559 ms 1.9086 ms 1.9689 ms]
col_mf/orx_arb_vv/e15_light time:   [1.7601 ms 1.7890 ms 1.8187 ms]

col_mf/seq/e20_light        time:   [3.0206 ms 3.0500 ms 3.0803 ms]
col_mf/rayon/e20_light      time:   [25.712 ms 26.450 ms 27.224 ms]
col_mf/rayon_ll/e20_light   time:   [25.877 ms 30.607 ms 37.267 ms]
col_mf/orx_ord/e20_light    time:   [4.1876 ms 4.2838 ms 4.3882 ms]
col_mf/orx_arb/e20_light    time:   [4.7258 ms 4.9768 ms 5.2453 ms]
col_mf/orx_arb_vv/e20_light time:   [3.3459 ms 3.5393 ms 3.7577 ms]

col_mf/seq/e15_heavy        time:   [956.76 µs 964.57 µs 973.31 µs]
col_mf/rayon/e15_heavy      time:   [15.471 ms 16.088 ms 16.751 ms]
col_mf/rayon_ll/e15_heavy   time:   [15.452 ms 16.072 ms 16.707 ms]
col_mf/orx_ord/e15_heavy    time:   [2.4677 ms 2.5068 ms 2.5472 ms]
col_mf/orx_arb/e15_heavy    time:   [2.3478 ms 2.3825 ms 2.4224 ms]
col_mf/orx_arb_vv/e15_heavy time:   [2.9152 ms 3.0580 ms 3.2042 ms]

col_mf/seq/e20_heavy        time:   [34.650 ms 35.484 ms 36.348 ms]
col_mf/rayon/e20_heavy      time:   [20.937 ms 22.609 ms 24.388 ms]
col_mf/rayon_ll/e20_heavy   time:   [26.650 ms 29.649 ms 33.218 ms]
col_mf/orx_ord/e20_heavy    time:   [9.9925 ms 10.807 ms 11.731 ms]
col_mf/orx_arb/e20_heavy    time:   [9.0371 ms 10.553 ms 12.642 ms]
col_mf/orx_arb_vv/e20_heavy time:   [7.5350 ms 7.7985 ms 8.1014 ms]

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
