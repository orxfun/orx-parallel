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

placeholder

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

fn orx<C: ParCollectInto<u64>>(input: &[u64], h: bool, order: IterationOrder) -> C {
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

fn rayon_ll(input: &[u64], h: bool) -> LinkedList<Vec<u64>> {
    match h {
        true => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(h_m2)
            .filter(f2)
            .collect_vec_list(),
        false => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(l_m2)
            .filter(f2)
            .collect_vec_list(),
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

    let mut group = c.benchmark_group("col_mfmf");

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
