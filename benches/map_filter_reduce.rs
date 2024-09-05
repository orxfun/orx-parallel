use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 98734;
const FIB_UPPER_BOUND: u32 = 999;

fn inputs(len: usize) -> Vec<usize> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.gen_range(0..FIB_UPPER_BOUND) as usize)
        .collect()
}

fn map(n: &usize) -> usize {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..*n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn filter(a: &usize) -> bool {
    a % 10 == 0 || a % 3 == 1
}

fn reduce(a: usize, b: usize) -> usize {
    a + b
}

fn seq(inputs: &[usize]) -> Option<usize> {
    inputs.iter().map(map).filter(filter).reduce(reduce)
}

fn rayon_reduce(inputs: &[usize]) -> Option<usize> {
    use rayon::iter::ParallelIterator;
    Some(
        inputs
            .into_par_iter()
            .map(map)
            .filter(filter)
            .reduce(|| 0, reduce),
    )
}

fn rayon_reduce_with(inputs: &[usize]) -> Option<usize> {
    use rayon::iter::ParallelIterator;
    inputs
        .into_par_iter()
        .map(map)
        .filter(filter)
        .reduce_with(reduce)
}

fn orx_parallel_custom_reduce(
    inputs: &[usize],
    num_threads: usize,
    chunk_size: usize,
) -> Option<usize> {
    inputs
        .iter()
        .cloned()
        .par()
        .num_threads(num_threads)
        .chunk_size(chunk_size)
        .reduce(reduce)
}

fn orx_parallel_sum(inputs: &[usize]) -> Option<usize> {
    Some(inputs.par().map(map).filter(filter).sum())
}

fn orx_parallel_reduce(inputs: &[usize]) -> Option<usize> {
    inputs.par().map(map).filter(filter).reduce(reduce)
}

fn orx_parallel_fold(inputs: &[usize]) -> Option<usize> {
    Some(inputs.par().map(map).filter(filter).fold(|| 0, reduce))
}

fn map_filter_reduce(c: &mut Criterion) {
    let lengths = [65_536, 65_536 * 4];
    let _params = [(1, 1), (8, 1024), (16, 1024), (32, 1024)];
    let params = [];

    let mut group = c.benchmark_group("map_filter_reduce");

    for len in lengths {
        let input = inputs(len);
        let name = format!("n{}", len);
        let expected = seq(&input);

        group.bench_with_input(BenchmarkId::new("seq", name.clone()), &name, |b, _| {
            b.iter(|| assert_eq!(expected, seq(black_box(&input))))
        });

        group.bench_with_input(
            BenchmarkId::new("rayon_reduce", name.clone()),
            &name,
            |b, _| b.iter(|| assert_eq!(expected, rayon_reduce(black_box(&input)))),
        );

        group.bench_with_input(
            BenchmarkId::new("rayon_reduce_with", name.clone()),
            &name,
            |b, _| b.iter(|| assert_eq!(expected, rayon_reduce_with(black_box(&input)))),
        );

        group.bench_with_input(
            BenchmarkId::new("orx-parallel-sum", name.clone()),
            &name,
            |b, _| b.iter(|| assert_eq!(expected, orx_parallel_sum(black_box(&input)))),
        );

        group.bench_with_input(
            BenchmarkId::new("orx-parallel-reduce", name.clone()),
            &name,
            |b, _| b.iter(|| assert_eq!(expected, orx_parallel_reduce(black_box(&input)))),
        );

        group.bench_with_input(
            BenchmarkId::new("orx-parallel-fold", name.clone()),
            &name,
            |b, _| b.iter(|| assert_eq!(expected, orx_parallel_fold(black_box(&input)))),
        );

        for (t, c) in params {
            let par_str = format!("orx-parallel-fold-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(par_str, name.clone()), &name, |b, _| {
                b.iter(|| {
                    assert_eq!(
                        expected,
                        orx_parallel_custom_reduce(black_box(&input), t, c)
                    )
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, map_filter_reduce);
criterion_main!(benches);
