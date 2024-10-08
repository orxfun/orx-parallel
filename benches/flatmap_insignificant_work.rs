use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 3134;

fn inputs(len: usize) -> Vec<u32> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.gen_range(0..9999)).collect()
}

fn map(a: &u32) -> Vec<usize> {
    [*a, a + 1, a + 2, a + 13]
        .map(|x| black_box(x as usize))
        .into_iter()
        .collect()
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

fn orx_parallel_vec(inputs: &[u32]) -> Vec<usize> {
    inputs.into_par().flat_map(map).collect_vec()
}

fn orx_parallel_split_rec(inputs: &[u32]) -> SplitVec<usize, Recursive> {
    inputs.into_par().flat_map(map).collect_x()
}

fn orx_parallel(inputs: &[u32], num_threads: usize, chunk_size: usize) -> Vec<usize> {
    inputs
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .flat_map(map)
        .collect_vec()
}

fn flat_map_insignificant_work(c: &mut Criterion) {
    let treatments = vec![65_536, 262_144 * 4];
    let _params = [(1, 1), (4, 256), (8, 1024), (32, 1024)];
    let params = [];

    let mut group = c.benchmark_group("flat_map_insignificant_work");

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
                b.iter(|| {
                    let result = orx_parallel(black_box(&input), t, c);
                    assert_eq!(result, expected);
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, flat_map_insignificant_work);
criterion_main!(benches);
