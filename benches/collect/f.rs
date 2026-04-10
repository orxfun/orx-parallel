/*

* eN means an input of size 2^N is used
* order:
  * _ord means results are collected in order leading to same result as sequential
  * _arb means results are collected in arbitrary order
* container:
  * _vec means, results are collected into a Vec
  * _vv means, results are collected into a Vec<Vec<_>>
  * _ll means, results are collected into a LinkedList<Vec<_>>

col_f/seq/e15           time:   [69.292 µs 70.238 µs 71.282 µs]
col_f/rayon/e15         time:   [16.038 ms 16.629 ms 17.270 ms]
col_f/rayon_ll/e15      time:   [15.302 ms 15.890 ms 16.539 ms]
col_f/orx_ord/e15       time:   [2.8782 ms 2.9945 ms 3.1132 ms]
col_f/orx_arb/e15       time:   [2.5767 ms 2.7047 ms 2.8337 ms]
col_f/orx_arb_vv/e15    time:   [1.7280 ms 1.7529 ms 1.7802 ms]

col_f/seq/e20           time:   [2.7309 ms 2.7623 ms 2.7940 ms]
col_f/rayon/e20         time:   [28.440 ms 30.007 ms 31.629 ms]
col_f/rayon_ll/e20      time:   [25.005 ms 26.317 ms 27.675 ms]
col_f/orx_ord/e20       time:   [9.4091 ms 9.9968 ms 10.675 ms]
col_f/orx_arb/e20       time:   [6.4440 ms 6.7267 ms 7.0154 ms]
col_f/orx_arb_vv/e20    time:   [4.3464 ms 4.6523 ms 5.0020 ms]

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
