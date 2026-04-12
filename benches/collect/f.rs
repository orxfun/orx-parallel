/*

* eN means an input of size 2^N is used
* order:
  * _ord means results are collected in order leading to same result as sequential
  * _arb means results are collected in arbitrary order
* container:
  * _vec means, results are collected into a Vec
  * _vv means, results are collected into a Vec<Vec<_>>
  * _ll means, results are collected into a LinkedList<Vec<_>>
  * Note that _ll and _vv 2-dim jagged results in rayon and orx, respectively

col_f/seq/e15           time:   [60.667 µs 61.717 µs 62.849 µs]
col_f/rayon/e15         time:   [14.201 ms 14.547 ms 14.903 ms]
col_f/rayon_ll/e15      time:   [14.382 ms 14.761 ms 15.153 ms]
col_f/orx_ord/e15       time:   [1.7878 ms 1.8191 ms 1.8513 ms]
col_f/orx_arb/e15       time:   [1.6353 ms 1.6602 ms 1.6858 ms]
col_f/orx_arb_vv/e15    time:   [1.6415 ms 1.6686 ms 1.7011 ms]

col_f/seq/e20           time:   [2.3798 ms 2.4067 ms 2.4345 ms]
col_f/rayon/e20         time:   [23.268 ms 23.956 ms 24.599 ms]
col_f/rayon_ll/e20      time:   [23.068 ms 23.831 ms 24.633 ms]
col_f/orx_ord/e20       time:   [4.0229 ms 4.1081 ms 4.1986 ms]
col_f/orx_arb/e20       time:   [4.3350 ms 4.4715 ms 4.6248 ms]
col_f/orx_arb_vv/e20    time:   [3.4061 ms 3.4698 ms 3.5362 ms]

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

fn f(a: &u64) -> bool {
    !(a + 7).is_multiple_of(11)
}

fn seq(input: &[u64]) -> Vec<u64> {
    input.iter().copied().filter(f).collect()
}

fn orx<C: ParCollectInto<u64>>(input: &[u64], order: IterationOrder) -> C {
    input
        .into_par()
        .iteration_order(order)
        .copied()
        .filter(f)
        .collect()
}

fn rayon(input: &[u64]) -> Vec<u64> {
    input.into_par_iter().copied().filter(f).collect()
}

fn rayon_ll(input: &[u64]) -> LinkedList<Vec<u64>> {
    input.into_par_iter().copied().filter(f).collect_vec_list()
}

struct Treat {
    len: usize,
}

fn run(c: &mut Criterion) {
    let treatments = [Treat { len: 1 << 15 }, Treat { len: 1 << 20 }];

    let mut group = c.benchmark_group("col_f");

    for t in treatments {
        let name = format!("e{}", t.len.ilog2(),);
        let input = inputs(t.len);
        let expected = seq(&input);
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
