use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 54487;

fn inputs(len: usize) -> Vec<usize> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.gen_range(0..15791)).collect()
}

fn map(a: usize) -> (usize, usize) {
    (a, 1)
}

fn red(a: (usize, usize), b: (usize, usize)) -> (usize, usize) {
    (a.0 + b.0, a.1 + b.1)
}

fn final_red(agg: (usize, usize)) -> usize {
    agg.0 / agg.1
}

fn seq(inputs: &[usize]) -> Option<usize> {
    inputs.iter().copied().map(map).reduce(red).map(final_red)
}

#[allow(clippy::unnecessary_map_on_constructor)]
fn rayon_reduce(inputs: &[usize]) -> Option<usize> {
    use rayon::iter::ParallelIterator;
    Some(
        inputs
            .into_par_iter()
            .copied()
            .map(map)
            .reduce(|| (0, 0), red),
    )
    .map(final_red)
}

fn rayon_reduce_with(inputs: &[usize]) -> Option<usize> {
    use rayon::iter::ParallelIterator;
    inputs
        .into_par_iter()
        .copied()
        .map(map)
        .reduce_with(red)
        .map(final_red)
}

fn orx_parallel_reduce(inputs: &[usize], num_threads: usize, chunk_size: usize) -> Option<usize> {
    inputs
        .iter()
        .cloned()
        .par()
        .num_threads(num_threads)
        .chunk_size(chunk_size)
        .map(map)
        .reduce(red)
        .map(final_red)
}

fn orx_parallel_sum(inputs: &[usize], num_threads: usize, chunk_size: usize) -> Option<usize> {
    Some(
        inputs
            .iter()
            .cloned()
            .par()
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .sum()
            / inputs.len(),
    )
}

fn orx_parallel_default(inputs: &[usize]) -> Option<usize> {
    Some(inputs.iter().cloned().par().sum() / inputs.len())
}

fn map_reduce_avg(c: &mut Criterion) {
    let lengths = [262_144 * 16];
    let params = [(1, 1), (4, 256), (8, 512), (8, 1024)];

    let mut group = c.benchmark_group("map_reduce_avg");

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

        for (t, c) in params {
            let params = format!("orx-parallel-sum-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(params, name.clone()), &name, |b, _| {
                b.iter(|| {
                    let result = orx_parallel_sum(black_box(&input), t, c);
                    assert_eq!(result, expected);
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, map_reduce_avg);
criterion_main!(benches);
