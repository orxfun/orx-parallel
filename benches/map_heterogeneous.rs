use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 745;
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

fn inputs(len: usize) -> Vec<u32> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.gen_range(0..FIB_UPPER_BOUND))
        .collect()
}

fn seq(inputs: &[u32]) -> Vec<u32> {
    inputs.iter().map(fibonacci).collect()
}

fn rayon(inputs: &[u32]) -> Vec<u32> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().map(fibonacci).collect()
}

fn orx_parallel(inputs: &[u32], num_threads: usize, chunk_size: usize) -> Vec<u32> {
    inputs
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .map(fibonacci)
        .collect_vec()
}

fn map_heterogeneous(c: &mut Criterion) {
    // let treatments = vec![65_536, 26 2_144];
    // let params = [(4, 64), (8, 64)];

    let treatments = vec![262_144 * 4];
    let params = [(8, 64)];

    let mut group = c.benchmark_group("map_heterogeneous");

    for n in &treatments {
        let input = inputs(*n);

        // group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
        //     b.iter(|| seq(black_box(&input)))
        // });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            b.iter(|| rayon(black_box(&input)))
        });

        for (t, c) in params {
            let name = format!("orx-parallel-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(name, n), n, |b, _| {
                b.iter(|| orx_parallel(black_box(&input), t, c))
            });
        }
    }

    group.finish();
}

criterion_group!(benches, map_heterogeneous);
criterion_main!(benches);
