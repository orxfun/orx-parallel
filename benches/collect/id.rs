/*

* _ord means results are collected in order consistent to input; _arb means order might be arbitrary
* eN means an input of size 2^N is used

col_id/seq/e15          time:   [8.2720 µs 8.3850 µs 8.4962 µs]
col_id/rayon/e15        time:   [11.827 ms 12.035 ms 12.247 ms]
col_id/orx_ord/e15      time:   [2.0194 ms 2.0907 ms 2.1727 ms]
col_id/orx_arb/e15      time:   [2.2263 ms 2.2872 ms 2.3505 ms]

col_id/seq/e20          time:   [569.69 µs 583.63 µs 601.98 µs]
col_id/rayon/e20        time:   [27.704 ms 28.970 ms 30.265 ms]
col_id/orx_ord/e20      time:   [8.6181 ms 8.9216 ms 9.2580 ms]
col_id/orx_arb/e20      time:   [12.039 ms 13.382 ms 15.112 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn seq(input: &[u64]) -> Vec<u64> {
    input.iter().copied().collect()
}

fn orx(input: &[u64], order: IterationOrder) -> Vec<u64> {
    input.into_par().iteration_order(order).copied().collect()
}

fn rayon(input: &[u64]) -> Vec<u64> {
    input.into_par_iter().copied().collect()
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

        group.bench_with_input(BenchmarkId::new("orx_ord", &name), &name, |b, _| {
            assert_eq!(&expected, &orx(&input, IterationOrder::Ordered));
            b.iter(|| orx(&input, IterationOrder::Ordered))
        });

        group.bench_with_input(BenchmarkId::new("orx_arb", &name), &name, |b, _| {
            let mut result = orx(&input, IterationOrder::Arbitrary);
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| orx(&input, IterationOrder::Ordered))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
