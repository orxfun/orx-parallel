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

col_id/seq/e15          time:   [5.7329 µs 5.7723 µs 5.8135 µs]
col_id/rayon/e15        time:   [8.2352 ms 8.3596 ms 8.4850 ms]
col_id/rayon_ll/e15     time:   [9.3046 ms 9.4592 ms 9.6156 ms]
col_id/orx_ord/e15      time:   [1.3669 ms 1.3814 ms 1.3970 ms]
col_id/orx_arb/e15      time:   [1.3063 ms 1.3221 ms 1.3374 ms]
col_id/orx_arb_vv/e15   time:   [1.3427 ms 1.3621 ms 1.3818 ms]

col_id/seq/e20          time:   [537.64 µs 543.36 µs 549.48 µs]
col_id/rayon/e20        time:   [18.018 ms 18.475 ms 18.938 ms]
col_id/rayon_ll/e20     time:   [17.159 ms 17.708 ms 18.269 ms]
col_id/orx_ord/e20      time:   [3.4113 ms 3.4772 ms 3.5469 ms]
col_id/orx_arb/e20      time:   [4.1789 ms 4.2580 ms 4.3391 ms]
col_id/orx_arb_vv/e20   time:   [2.7635 ms 2.8331 ms 2.9044 ms]

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
