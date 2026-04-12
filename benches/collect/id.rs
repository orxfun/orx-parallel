/*

* eN means an input of size 2^N is used
* order:
  * _ord means results are collected in order leading to same result as sequential
  * _arb means results are collected in arbitrary order
* container:
  * _vec means, results are collected into a Vec
  * _vv means, results are collected into a Vec<Vec<_>>
  * _ll means, results are collected into a LinkedList<Vec<_>>
  * Note that _vv and _ll are corresponding jagged results in rayon and orx

col_id/seq/e15          time:   [6.8940 µs 6.9559 µs 7.0233 µs]
col_id/rayon/e15        time:   [11.482 ms 11.940 ms 12.399 ms]
col_id/rayon_ll/e15     time:   [13.386 ms 14.483 ms 15.816 ms]
col_id/orx_ord/e15      time:   [2.2033 ms 2.3224 ms 2.4529 ms]
col_id/orx_arb/e15      time:   [2.6000 ms 2.7021 ms 2.8091 ms]
col_id/orx_arb_vv/e15   time:   [2.0926 ms 2.2088 ms 2.3356 ms]

col_id/seq/e20          time:   [687.07 µs 715.43 µs 741.66 µs]
col_id/rayon/e20        time:   [27.934 ms 31.761 ms 37.343 ms]
col_id/rayon_ll/e20     time:   [21.114 ms 22.364 ms 23.707 ms]
col_id/orx_ord/e20      time:   [10.583 ms 11.717 ms 13.284 ms]
col_id/orx_arb/e20      time:   [5.2519 ms 5.7780 ms 6.5096 ms]
col_id/orx_arb_vv/e20   time:   [2.9987 ms 3.0896 ms 3.1825 ms]

// TODO: the difference between orx_ord and orx_arb is due to post-ordering, which can be improved

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

fn seq(input: &[u64]) -> Vec<u64> {
    input.iter().copied().collect()
}

fn orx<C: ParCollectInto<u64>>(input: &[u64], order: IterationOrder) -> C {
    input.into_par().iteration_order(order).copied().collect()
}

fn rayon(input: &[u64]) -> Vec<u64> {
    input.into_par_iter().copied().collect()
}

fn rayon_ll(input: &[u64]) -> LinkedList<Vec<u64>> {
    input.into_par_iter().copied().collect_vec_list()
}

struct Treat {
    len: usize,
}

fn run(c: &mut Criterion) {
    let treatments = [Treat { len: 1 << 15 }, Treat { len: 1 << 20 }];

    let mut group = c.benchmark_group("col_id");

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
