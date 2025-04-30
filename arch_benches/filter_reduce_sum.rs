use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 846356;

fn inputs(len: usize) -> Vec<usize> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..15791)).collect()
}

fn red(a: usize, b: usize) -> usize {
    a + b
}

fn fil(a: &usize) -> bool {
    let a = *a;
    a < 26 || a % 3 == 1 || (265..657).contains(&a) || a % 2 == 0
}

fn seq(inputs: &[usize]) -> Option<usize> {
    inputs.iter().copied().filter(fil).reduce(red)
}

fn rayon_reduce(inputs: &[usize]) -> Option<usize> {
    use rayon::iter::ParallelIterator;
    Some(
        inputs
            .into_par_iter()
            .copied()
            .filter(fil)
            .reduce(|| 0, red),
    )
}

fn rayon_reduce_with(inputs: &[usize]) -> Option<usize> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().copied().filter(fil).reduce_with(red)
}

fn orx_parallel_default(inputs: &[usize]) -> Option<usize> {
    inputs.iter().cloned().par().filter(fil).reduce(red)
}

fn orx_parallel(inputs: &[usize], num_threads: usize, chunk_size: usize) -> Option<usize> {
    inputs
        .iter()
        .cloned()
        .par()
        .num_threads(num_threads)
        .chunk_size(chunk_size)
        .filter(fil)
        .reduce(red)
}

fn filter_reduce_sum(c: &mut Criterion) {
    let lengths = [262_144 * 4];
    let params = [(1, 1), (4, 256), (8, 512), (8, 1024)];

    let mut group = c.benchmark_group("filter_reduce_sum");

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

        for (t, c) in params {
            let params = format!("orx-parallel-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(params, name.clone()), &name, |b, _| {
                b.iter(|| {
                    let result = orx_parallel(black_box(&input), t, c);
                    assert_eq!(result, expected);
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, filter_reduce_sum);
criterion_main!(benches);
