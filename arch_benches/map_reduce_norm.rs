use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

const SEED: u64 = 12146;

fn inputs(len: usize) -> Vec<i64> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..104)).collect()
}

fn seq(inputs: &[i64]) -> f64 {
    let sum = inputs.iter().map(|&x| x * x).sum::<i64>();
    (sum as f64).sqrt()
}

fn rayon(inputs: &[i64]) -> f64 {
    let sum = inputs.par_iter().map(|&x| x * x).sum::<i64>();
    (sum as f64).sqrt()
}

fn orx_parallel(inputs: &[i64], num_threads: usize, chunk_size: usize) -> f64 {
    let sum = inputs
        .into_par()
        .num_threads(num_threads)
        .chunk_size(chunk_size)
        .map(|&x| x * x)
        .sum();
    (sum as f64).sqrt()
}

fn orx_parallel_default(inputs: &[i64]) -> f64 {
    let sum = inputs.into_par().map(|&x| x * x).sum();
    (sum as f64).sqrt()
}

fn map_reduce_norm(c: &mut Criterion) {
    let lengths = [262_144 * 16];
    let params = [(1, 1), (4, 256), (8, 512), (8, 1024)];

    let mut group = c.benchmark_group("map_reduce_norm");

    for len in lengths {
        let input = inputs(len);
        let name = format!("n{}", len);
        let expected = seq(&input);

        group.bench_with_input(BenchmarkId::new("seq", name.clone()), &name, |b, _| {
            b.iter(|| {
                let result = seq(black_box(&input));
                assert!((result - expected).abs() < 0.1);
            })
        });

        group.bench_with_input(BenchmarkId::new("rayon", name.clone()), &name, |b, _| {
            b.iter(|| {
                let result = rayon(black_box(&input));
                assert!((result - expected).abs() < 0.1);
            })
        });

        group.bench_with_input(
            BenchmarkId::new("orx-parallel-default", name.clone()),
            &name,
            |b, _| {
                b.iter(|| {
                    let result = orx_parallel_default(black_box(&input));
                    assert!((result - expected).abs() < 0.1);
                })
            },
        );

        for (t, c) in params {
            let params = format!("orx-parallel-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(params, name.clone()), &name, |b, _| {
                b.iter(|| {
                    let result = orx_parallel(black_box(&input), t, c);
                    assert!((result - expected).abs() < 0.1);
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, map_reduce_norm);
criterion_main!(benches);
