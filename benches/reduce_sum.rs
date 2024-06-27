use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_concurrent_iter::*;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 54487;

fn inputs(len: usize) -> Vec<usize> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.gen_range(0..15791)).collect()
}

fn red(a: usize, b: usize) -> usize {
    a + b
}

fn seq(inputs: &[usize]) -> Option<usize> {
    inputs.iter().copied().reduce(red)
}

fn rayon_reduce(inputs: &[usize]) -> Option<usize> {
    use rayon::iter::ParallelIterator;
    Some(inputs.into_par_iter().copied().reduce(|| 0, red))
}

fn rayon_reduce_with(inputs: &[usize]) -> Option<usize> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().copied().reduce_with(red)
}

fn orx_parallel_reduce(inputs: &[usize], num_threads: usize, chunk_size: usize) -> Option<usize> {
    inputs
        .into_con_iter()
        .cloned()
        .into_par()
        .num_threads(num_threads)
        .chunk_size(chunk_size)
        .reduce(red)
}

fn orx_parallel_fold(inputs: &[usize], num_threads: usize, chunk_size: usize) -> Option<usize> {
    Some(
        inputs
            .into_con_iter()
            .cloned()
            .into_par()
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .fold(|| 0, red),
    )
}

fn orx_parallel_sum(inputs: &[usize], num_threads: usize, chunk_size: usize) -> Option<usize> {
    Some(
        inputs
            .into_con_iter()
            .cloned()
            .into_par()
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .sum(),
    )
}

fn orx_parallel_default(inputs: &[usize]) -> Option<usize> {
    Some(inputs.into_con_iter().cloned().into_par().sum())
}

fn reduce_sum(c: &mut Criterion) {
    let lengths = [262_144 * 16];
    let params = [(1, 1), (8, 1024), (16, 1024), (32, 1024)];

    let mut group = c.benchmark_group("reduce_sum");

    for len in lengths {
        let input = inputs(len);
        let name = format!("n{}", len);
        let expected = seq(&input);

        group.bench_with_input(BenchmarkId::new("seq", name.clone()), &name, |b, _| {
            b.iter(|| {
                let result = seq(black_box(&input));
                assert_eq!(result, expected);
            })
        });

        group.bench_with_input(
            BenchmarkId::new("rayon_reduce", name.clone()),
            &name,
            |b, _| {
                b.iter(|| {
                    let result = rayon_reduce(black_box(&input));
                    assert_eq!(result, expected);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rayon_reduce_with", name.clone()),
            &name,
            |b, _| {
                b.iter(|| {
                    let result = rayon_reduce_with(black_box(&input));
                    assert_eq!(result, expected);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("orx-parallel-default", name.clone()),
            &name,
            |b, _| {
                b.iter(|| {
                    let result = orx_parallel_default(black_box(&input));
                    assert_eq!(result, expected);
                })
            },
        );

        let t = 8;
        let c = 1024;
        let par_str = format!("orx-parallel-reduce-t{}-c{}", t, c);
        group.bench_with_input(BenchmarkId::new(par_str, name.clone()), &name, |b, _| {
            b.iter(|| {
                let result = orx_parallel_reduce(black_box(&input), t, c);
                assert_eq!(result, expected);
            })
        });

        let par_str = format!("orx-parallel-fold-t{}-c{}", t, c);
        group.bench_with_input(BenchmarkId::new(par_str, name.clone()), &name, |b, _| {
            b.iter(|| {
                let result = orx_parallel_fold(black_box(&input), t, c);
                assert_eq!(result, expected);
            })
        });

        for (t, c) in params {
            let par_str = format!("orx-parallel-sum-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(par_str, name.clone()), &name, |b, _| {
                b.iter(|| {
                    let result = orx_parallel_sum(black_box(&input), t, c);
                    assert_eq!(result, expected);
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, reduce_sum);
criterion_main!(benches);
