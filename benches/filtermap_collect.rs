use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 745;
const FIB_UPPER_BOUND: u32 = 9999;

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

fn filter_map(n: &u32) -> Option<u32> {
    let value = fibonacci(n);
    match value % 5 == 0 {
        true => None,
        false => Some(n + 1),
    }
}

fn inputs(len: usize) -> Vec<u32> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.gen_range(0..FIB_UPPER_BOUND))
        .collect()
}

fn seq(inputs: &[u32]) -> Vec<u32> {
    inputs.iter().filter_map(filter_map).collect()
}

fn rayon(inputs: &[u32]) -> Vec<u32> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().filter_map(filter_map).collect()
}

fn orx_parallel_default_chain(inputs: &[u32]) -> Vec<u32> {
    inputs
        .into_par()
        .map(filter_map)
        .filter(|x| x.is_some())
        .map(|x| x.expect("is-some"))
        .collect_vec()
}

fn orx_parallel_default(inputs: &[u32]) -> Vec<u32> {
    inputs.into_par().filter_map(filter_map).collect_vec()
}

fn orx_parallel(inputs: &[u32], num_threads: usize, chunk_size: usize) -> Vec<u32> {
    inputs
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .filter_map(filter_map)
        .collect_vec()
}

fn filtermap_collect(c: &mut Criterion) {
    let treatments = vec![65_536, 262_144 * 4];
    let params = [(1, 1), (4, 256), (8, 1024), (32, 1024)];

    let mut group = c.benchmark_group("filtermap_collect");

    for n in &treatments {
        let input = inputs(*n);
        let expected = seq(&input);

        group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
            b.iter(|| seq(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            b.iter(|| {
                let result = rayon(black_box(&input));
                assert_eq!(result, expected);
            })
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-default", n), n, |b, _| {
            b.iter(|| {
                let result = orx_parallel_default(black_box(&input));
                assert_eq!(result, expected);
            })
        });

        group.bench_with_input(
            BenchmarkId::new("orx-parallel-default-chain", n),
            n,
            |b, _| {
                b.iter(|| {
                    let result = orx_parallel_default_chain(black_box(&input));
                    assert_eq!(result, expected);
                })
            },
        );

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

criterion_group!(benches, filtermap_collect);
criterion_main!(benches);
