use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 9874;
const FIB_UPPER_BOUND: u32 = 999;

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

fn filter(a: &u32) -> bool {
    a % 10 == 0 || a % 3 != 1
}

fn inputs(len: usize) -> Vec<u32> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.gen_range(0..FIB_UPPER_BOUND))
        .collect()
}

fn seq(inputs: &[u32]) -> Vec<u32> {
    inputs.iter().map(fibonacci).filter(filter).collect()
}

fn rayon(inputs: &[u32]) -> Vec<u32> {
    use rayon::iter::ParallelIterator;
    inputs
        .into_par_iter()
        .map(fibonacci)
        .filter(filter)
        .collect()
}

fn orx_parallel_split_vec(inputs: &[u32]) -> SplitVec<u32> {
    inputs.into_par().map(fibonacci).filter(filter).collect()
}

fn orx_parallel_split_rec(inputs: &[u32]) -> SplitVec<u32, Recursive> {
    inputs.into_par().map(fibonacci).filter(filter).collect_x()
}

fn orx_parallel_vec(inputs: &[u32]) -> Vec<u32> {
    inputs
        .into_par()
        .map(fibonacci)
        .filter(filter)
        .collect_vec()
}

fn orx_parallel(inputs: &[u32], num_threads: usize, chunk_size: usize) -> Vec<u32> {
    inputs
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .map(fibonacci)
        .filter(filter)
        .collect_vec()
}

fn map_filter_collect(c: &mut Criterion) {
    let treatments = vec![65_536, 262_144];
    let _params = [(1, 1), (4, 256), (8, 1024), (32, 1024)];
    let params = [];

    let mut group = c.benchmark_group("map_filter_collect");

    for n in &treatments {
        let input = inputs(*n);
        let expected = seq(&input);

        group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
            b.iter(|| seq(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            assert_eq!(rayon(&input), expected);
            b.iter(|| rayon(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-split-vec", n), n, |b, _| {
            assert_eq!(orx_parallel_split_vec(&input), expected);
            b.iter(|| orx_parallel_split_vec(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-vec", n), n, |b, _| {
            assert_eq!(orx_parallel_vec(&input), expected);
            b.iter(|| orx_parallel_vec(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-split-rec", n), n, |b, _| {
            let mut result = orx_parallel_split_rec(&input).to_vec();
            result.sort();
            let mut expected = expected.clone();
            expected.sort();
            assert_eq!(result, expected);
            b.iter(|| orx_parallel_split_rec(black_box(&input)))
        });

        for (t, c) in params {
            let name = format!("orx-parallel-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(name, n), n, |b, _| {
                assert_eq!(orx_parallel(&input, t, c), expected);
                b.iter(|| orx_parallel(black_box(&input), t, c))
            });
        }
    }

    group.finish();
}

criterion_group!(benches, map_filter_collect);
criterion_main!(benches);
