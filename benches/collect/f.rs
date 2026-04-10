/*

* eN means an input of size 2^N is used
* order:
  * _ord means results are collected in order leading to same result as sequential
  * _arb means results are collected in arbitrary order
* container:
  * _vec means, results are collected into a Vec
  * _vv means, results are collected into a Vec<Vec<_>>
  * _ll means, results are collected into a LinkedList<Vec<_>>

col_f/seq/e15           time:   [66.363 µs 67.266 µs 68.192 µs]
col_f/rayon/e15         time:   [12.274 ms 12.614 ms 12.946 ms]
col_f/rayon_ll/e15      time:   [13.592 ms 13.895 ms 14.203 ms]
col_f/orx_ord/e15       time:   [2.0706 ms 2.1105 ms 2.1536 ms]
col_f/orx_arb/e15       time:   [2.7902 ms 3.1102 ms 3.4431 ms]
col_f/orx_arb_rec/e15   time:   [1.8972 ms 1.9430 ms 1.9899 ms]

col_f/seq/e20           time:   [2.7173 ms 2.7437 ms 2.7711 ms]
col_f/rayon/e20         time:   [23.614 ms 24.375 ms 25.201 ms]
col_f/rayon_ll/e20      time:   [24.258 ms 27.253 ms 31.696 ms]
col_f/orx_ord/e20       time:   [6.6624 ms 6.7479 ms 6.8349 ms]
col_f/orx_arb/e20       time:   [3.9299 ms 4.0616 ms 4.2114 ms]
col_f/orx_arb_rec/e20   time:   [3.2994 ms 3.4018 ms 3.5159 ms]

*/

use std::collections::LinkedList;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
use orx_split_vec::{IntoFragments, Recursive, SplitVec};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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
            assert_eq!(&expected, &orx::<Vec<_>>(&input, IterationOrder::Ordered));
            b.iter(|| orx::<Vec<_>>(&input, IterationOrder::Ordered))
        });

        group.bench_with_input(BenchmarkId::new("orx_arb", &name), &name, |b, _| {
            let mut result: Vec<_> = orx(&input, IterationOrder::Arbitrary);
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| orx::<Vec<_>>(&input, IterationOrder::Arbitrary))
        });

        group.bench_with_input(BenchmarkId::new("orx_arb_vv", &name), &name, |b, _| {
            let mut result: Vec<u64> = orx::<Vec<Vec<_>>>(&input, IterationOrder::Arbitrary)
                .flat_map(|x| x.into_iter())
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
