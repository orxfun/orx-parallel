use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;
use std::num::NonZeroUsize;

const SEED: u64 = 745;
const FIB_UPPER_BOUND: u32 = 9999;

fn fibonacci(n: &u32) -> u32 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..*n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn map(n: &u32) -> u32 {
    let sum: u32 = (0u32..1000).map(|_| black_box(fibonacci(n))).sum();
    sum.min(*n)
}

fn predicate(a: &u32) -> bool {
    a % 64 == 0
}

fn inputs(len: usize) -> Vec<u32> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.gen_range(0..FIB_UPPER_BOUND))
        .collect()
}

fn seq(inputs: &[u32]) -> Option<u32> {
    inputs.iter().map(map).find(predicate)
}

fn rayon(inputs: &[u32]) -> Option<u32> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().map(map).find_first(predicate)
}

fn orx_parallel_default(inputs: &[u32]) -> Option<u32> {
    inputs.into_par().map(map).find(predicate)
}

fn orx_parallel(inputs: &[u32], num_threads: usize, chunk_size: usize) -> Option<u32> {
    inputs
        .into_par()
        .chunk_size(ChunkSize::Exact(NonZeroUsize::new(chunk_size).unwrap()))
        .num_threads(num_threads)
        .map(map)
        .find(predicate)
}

fn map_find_expensive(c: &mut Criterion) {
    let treatments = vec![65_536, 262_144 * 4];
    let params = [(32, 1), (32, 4)];

    let mut group = c.benchmark_group("map_find_expensive");

    for n in &treatments {
        let input = inputs(*n);
        let expected = seq(&input);

        group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
            b.iter(|| seq(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            b.iter(|| {
                let result = rayon(black_box(&input));
                assert_eq!(result, expected);
            })
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-default", n), n, |b, _| {
            b.iter(|| {
                let result = orx_parallel_default(black_box(&input));
                assert_eq!(result, expected);
            })
        });

        for (t, c) in params {
            let name = format!("orx-parallel-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(name, n), n, |b, _| {
                b.iter(|| {
                    let result = orx_parallel(black_box(&input), t, c);
                    assert_eq!(result, expected);
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, map_find_expensive);
criterion_main!(benches);
