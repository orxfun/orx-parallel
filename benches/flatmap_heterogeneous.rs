use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use orx_split_vec::{Recursive, SplitVec};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 3134;
const FIB_UPPER_BOUND: u32 = 999;

fn fibonacci(n: u32) -> usize {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a as usize
}

fn inputs(len: usize) -> Vec<u32> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.gen_range(0..FIB_UPPER_BOUND))
        .collect()
}

fn map(a: &u32) -> Vec<usize> {
    let len = a % 100;
    (0..len).map(fibonacci).into_iter().collect()
}

fn seq(inputs: &[u32]) -> Vec<usize> {
    inputs.iter().flat_map(map).collect()
}

fn rayon(inputs: &[u32]) -> Vec<usize> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().flat_map(map).collect()
}

fn orx_parallel_split_vec(inputs: &[u32]) -> SplitVec<usize> {
    inputs.into_par().flat_map(map).collect()
}

fn orx_parallel_split_rec(inputs: &[u32]) -> SplitVec<usize, Recursive> {
    inputs.into_par().flat_map(map).collect_x()
}

fn orx_parallel_vec(inputs: &[u32]) -> Vec<usize> {
    inputs.into_par().flat_map(map).collect_vec()
}

fn orx_parallel(inputs: &[u32], num_threads: usize, chunk_size: usize) -> Vec<usize> {
    inputs
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .flat_map(map)
        .collect_vec()
}

fn flatmap_heterogeneous(c: &mut Criterion) {
    let treatments = vec![65_536, 262_144 * 4];
    let _params = [(1, 1), (4, 256), (8, 1024), (32, 1024)];
    let params = [];

    let mut group = c.benchmark_group("flatmap_heterogeneous");

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
                assert_eq!(orx_parallel(black_box(&input), t, c), expected);
                b.iter(|| orx_parallel(black_box(&input), t, c))
            });
        }
    }

    group.finish();
}

criterion_group!(benches, flatmap_heterogeneous);
criterion_main!(benches);
