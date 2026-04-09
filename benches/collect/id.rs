/*

* _ord means results are collected in order consistent to input; _arb means order might be arbitrary
* eN means an input of size 2^N is used

reduce_id/seq/e15_light     time:   [4.7806 µs 4.8926 µs 4.9974 µs]
reduce_id/rayon1/e15_light  time:   [8.4413 ms 8.7003 ms 8.9418 ms]
reduce_id/rayon2/e15_light  time:   [9.7850 ms 9.9723 ms 10.161 ms]
reduce_id/orx/e15_light     time:   [1.2602 ms 1.2748 ms 1.2906 ms]

reduce_id/seq/e20_light     time:   [232.11 µs 234.04 µs 236.05 µs]
reduce_id/rayon1/e20_light  time:   [17.857 ms 18.285 ms 18.712 ms]
reduce_id/rayon2/e20_light  time:   [17.781 ms 18.538 ms 19.279 ms]
reduce_id/orx/e20_light     time:   [2.0632 ms 2.0930 ms 2.1245 ms]

reduce_id/seq/e15_heavy     time:   [1.4879 ms 1.5022 ms 1.5188 ms]
reduce_id/rayon1/e15_heavy  time:   [10.655 ms 10.904 ms 11.147 ms]
reduce_id/rayon2/e15_heavy  time:   [10.854 ms 11.052 ms 11.250 ms]
reduce_id/orx/e15_heavy     time:   [2.2717 ms 2.3045 ms 2.3441 ms]

reduce_id/seq/e20_heavy     time:   [49.829 ms 50.799 ms 51.830 ms]
reduce_id/rayon1/e20_heavy  time:   [13.010 ms 14.283 ms 15.763 ms]
reduce_id/rayon2/e20_heavy  time:   [15.656 ms 16.773 ms 18.057 ms]
reduce_id/orx/e20_heavy     time:   [7.8542 ms 8.1934 ms 8.6432 ms]

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
