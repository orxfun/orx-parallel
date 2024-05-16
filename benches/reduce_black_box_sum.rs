use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_concurrent_iter::IntoConcurrentIter;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 846356;

fn inputs(len: usize) -> Vec<usize> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.gen_range(0..15791)).collect()
}

fn red(a: usize, b: usize) -> usize {
    black_box(black_box(a) + black_box(b))
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

fn orx_parallel(inputs: &[usize], num_threads: usize) -> Option<usize> {
    inputs
        .into_con_iter()
        .cloned()
        .into_par()
        .num_threads(num_threads)
        .reduce(red)
}

fn reduce_black_box_sum(c: &mut Criterion) {
    let lengths = [262_144 * 16];
    let params = [8];

    let mut group = c.benchmark_group("reduce_black_box_sum");

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

        // group.bench_with_input(
        //     BenchmarkId::new("rayon_reduce_with", name.clone()),
        //     &name,
        //     |b, _| {
        //         b.iter(|| {
        //             let result = rayon_reduce_with(black_box(&input));
        //             assert_eq!(result, expected);
        //         })
        //     },
        // );

        for t in params {
            let params = format!("orx-parallel-t{}", t);
            group.bench_with_input(BenchmarkId::new(params, name.clone()), &name, |b, _| {
                b.iter(|| {
                    let result = orx_parallel(black_box(&input), t);
                    assert_eq!(result, expected);
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, reduce_black_box_sum);
criterion_main!(benches);
