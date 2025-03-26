use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 2356;
const FIB_UPPER_BOUND: u32 = 99;

#[allow(dead_code)]
#[derive(Debug)]
struct LargeOutput {
    name: String,
    numbers: [i64; 64],
    vectors: Vec<Vec<i64>>,
}

// todo! why do we need Default?
impl Default for LargeOutput {
    fn default() -> Self {
        Self {
            name: Default::default(),
            numbers: [0; 64],
            vectors: Default::default(),
        }
    }
}

fn to_large_output(idx: &usize) -> LargeOutput {
    let idx = *idx;
    let prefix = match idx % 7 {
        0 => "zero-",
        3 => "three-",
        _ => "sth-",
    };
    let fib = fibonacci(&(idx as u32));
    let name = format!("{}-fib-{}", prefix, fib);

    let mut numbers = [0i64; 64];
    for (i, x) in numbers.iter_mut().enumerate() {
        *x = match (idx * 7 + i) % 3 {
            0 => idx as i64 + i as i64,
            _ => idx as i64 - i as i64,
        };
    }

    let mut vectors = vec![];
    for i in 0..8 {
        let mut vec = vec![];
        for j in 0..idx % 1024 {
            vec.push(idx as i64 - i as i64 + j as i64);
        }
        vectors.push(vec);
    }

    LargeOutput {
        name,
        numbers,
        vectors,
    }
}

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

fn inputs(len: usize) -> Vec<usize> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.gen_range(0..FIB_UPPER_BOUND) as usize)
        .collect()
}

fn seq(inputs: &[usize]) -> Vec<LargeOutput> {
    inputs.iter().map(to_large_output).collect()
}

fn rayon(inputs: &[usize]) -> Vec<LargeOutput> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().map(to_large_output).collect()
}

fn orx_parallel_default(inputs: &[usize]) -> Vec<LargeOutput> {
    inputs.into_par().map(to_large_output).collect_vec()
}

fn orx_parallel(inputs: &[usize], num_threads: usize, chunk_size: usize) -> Vec<LargeOutput> {
    inputs
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .map(to_large_output)
        .collect_vec()
}

fn map_to_large_type(c: &mut Criterion) {
    let treatments = vec![65_536];
    let params = [(1, 1), (4, 256), (8, 512), (8, 1024)];

    let mut group = c.benchmark_group("map_to_large_type");

    for n in &treatments {
        let input = inputs(*n);

        group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
            b.iter(|| seq(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            b.iter(|| rayon(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-default", n), n, |b, _| {
            b.iter(|| orx_parallel_default(black_box(&input)))
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

criterion_group!(benches, map_to_large_type);
criterion_main!(benches);
