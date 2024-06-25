use criterion::{black_box, criterion_group, criterion_main, Criterion};
use orx_parallel::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

const LEN_1: usize = 512;
const LEN_2: usize = 512;
const LEN_3: usize = 16;

fn seq() -> usize {
    (0..LEN_1)
        .map(|x| {
            (0..LEN_2)
                .map(|y| (0..LEN_3).map(|z| black_box(x * y * z)).sum::<usize>())
                .sum::<usize>()
        })
        .sum::<usize>()
}

fn rayon_1() -> usize {
    (0..LEN_1)
        .into_par_iter()
        .map(|x| {
            (0..LEN_2)
                .map(|y| (0..LEN_3).map(|z| black_box(x * y * z)).sum::<usize>())
                .sum::<usize>()
        })
        .sum::<usize>()
}

fn rayon_12() -> usize {
    (0..LEN_1)
        .into_par_iter()
        .map(|x| {
            (0..LEN_2)
                .into_par_iter()
                .map(|y| (0..LEN_3).map(|z| black_box(x * y * z)).sum::<usize>())
                .sum::<usize>()
        })
        .sum::<usize>()
}

fn orx_parallel_1(num_threads: usize, chunk_size: usize) -> usize {
    (0..LEN_1)
        .par()
        .num_threads(num_threads)
        .chunk_size(chunk_size)
        .map(|x| {
            (0..LEN_2)
                .map(|y| (0..LEN_3).map(|z| black_box(x * y * z)).sum::<usize>())
                .sum::<usize>()
        })
        .sum()
}

fn orx_parallel_defaults_1() -> usize {
    (0..LEN_1)
        .par()
        .map(|x| {
            (0..LEN_2)
                .map(|y| (0..LEN_3).map(|z| black_box(x * y * z)).sum::<usize>())
                .sum::<usize>()
        })
        .sum()
}

fn orx_parallel_defaults_12() -> usize {
    (0..LEN_1)
        .par()
        .map(|x| {
            (0..LEN_2)
                .par()
                .map(|y| (0..LEN_3).map(|z| black_box(x * y * z)).sum::<usize>())
                .sum()
        })
        .sum()
}

fn orx_parallel_12(num_threads: usize, chunk_size: usize) -> usize {
    (0..LEN_1)
        .par()
        .num_threads(num_threads)
        .chunk_size(chunk_size)
        .map(|x| {
            (0..LEN_2)
                .par()
                .num_threads(num_threads)
                .chunk_size(chunk_size)
                .map(|y| (0..LEN_3).map(|z| black_box(x * y * z)).sum())
                .sum()
        })
        .sum()
}

fn nested_reduction(c: &mut Criterion) {
    let params = [(1, 1), (8, 32), (8, 64)];

    let mut group = c.benchmark_group("nested_reduction");

    group.bench_function("seq", |b| b.iter(seq));

    group.bench_function("rayon_1", |b| b.iter(rayon_1));
    group.bench_function("rayon_12", |b| b.iter(rayon_12));

    group.bench_function("orx-parallel-defaults-1", |b| {
        b.iter(orx_parallel_defaults_1)
    });

    group.bench_function("orx-parallel-defaults-12", |b| {
        b.iter(orx_parallel_defaults_12)
    });

    for (t, c) in params {
        let name = format!("orx-parallel-1-t{}-c{}", t, c);
        group.bench_function(name, |b| b.iter(|| orx_parallel_1(t, c)));

        let name = format!("orx-parallel-12-t{}-c{}", t, c);
        group.bench_function(name, |b| b.iter(|| orx_parallel_12(t, c)));
    }

    group.finish();
}

criterion_group!(benches, nested_reduction);
criterion_main!(benches);
