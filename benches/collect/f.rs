/*

* eN means an input of size 2^N is used
* _ord means results are collected in order consistent to input
* _arb means order might be arbitrary
* _arb_rec means the results are collected in arbitrary order into a SplitVec<_, Recursive>,
  which can be converted into Vec<Vec<_>>

col_f/seq/e15           time:   [73.176 µs 74.413 µs 75.712 µs]
col_f/rayon/e15         time:   [14.387 ms 14.810 ms 15.240 ms]
col_f/orx_ord/e15       time:   [2.2811 ms 2.3480 ms 2.4199 ms]
col_f/orx_arb/e15       time:   [2.3316 ms 2.4264 ms 2.5335 ms]

col_f/seq/e20           time:   [3.4547 ms 3.5284 ms 3.6010 ms]
col_f/rayon/e20         time:   [31.111 ms 32.851 ms 34.651 ms]
col_f/orx_ord/e20       time:   [8.7023 ms 8.9343 ms 9.1709 ms]
col_f/orx_arb/e20       time:   [8.9478 ms 9.2851 ms 9.6430 ms]

*/

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

fn orx(input: &[u64], order: IterationOrder) -> Vec<u64> {
    input
        .into_par()
        .iteration_order(order)
        .copied()
        .filter(f)
        .collect()
}

fn orx_arb_rec(input: &[u64]) -> SplitVec<u64, Recursive> {
    input.into_par().copied().filter(f).collect()
}

fn rayon(input: &[u64]) -> Vec<u64> {
    input.into_par_iter().copied().filter(f).collect()
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

        group.bench_with_input(BenchmarkId::new("orx_ord", &name), &name, |b, _| {
            assert_eq!(&expected, &orx(&input, IterationOrder::Ordered));
            b.iter(|| orx(&input, IterationOrder::Ordered))
        });

        group.bench_with_input(BenchmarkId::new("orx_arb", &name), &name, |b, _| {
            let mut result = orx(&input, IterationOrder::Arbitrary);
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| orx(&input, IterationOrder::Arbitrary))
        });

        group.bench_with_input(BenchmarkId::new("orx_arb_rec", &name), &name, |b, _| {
            let mut result: Vec<u64> = orx_arb_rec(&input)
                .into_fragments()
                .flat_map(|x| Vec::from(x).into_iter())
                .collect();
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| orx_arb_rec(&input))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
