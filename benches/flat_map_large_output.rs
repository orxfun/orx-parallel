use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;
use std::num::NonZeroUsize;

const SEED: u64 = 89774;
const FIB_UPPER_BOUND: u32 = 30;

#[derive(PartialEq, Eq, Debug)]
struct LargeOutput {
    idx: usize,
    name: String,
    numbers: [i64; 64],
    vectors: Vec<Vec<i64>>,
}

// todo! why do we need Default?
impl Default for LargeOutput {
    fn default() -> Self {
        Self {
            idx: 0,
            name: Default::default(),
            numbers: [0; 64],
            vectors: Default::default(),
        }
    }
}

fn to_large_output(idx: u32) -> LargeOutput {
    let idx = idx as usize;
    let prefix = match idx % 7 {
        0 => "zero-",
        3 => "three-",
        _ => "sth-",
    };
    let fib = fibonacci(idx as u32 % 50);
    let name = format!("{}-fib-{}", prefix, fib);

    let mut numbers = [0i64; 64];
    for (i, x) in numbers.iter_mut().enumerate() {
        *x = match (idx * 7 + i) % 3 {
            0 => idx as i64 + i as i64,
            1 => idx as i64 - i as i64,
            _ => fibonacci((idx % 30) as u32) as i64,
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
        idx,
        name,
        numbers,
        vectors,
    }
}

impl PartialOrd for LargeOutput {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
}
impl Ord for LargeOutput {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}

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

fn fmap(a: &u32) -> Vec<LargeOutput> {
    [*a, a + 1, a + 2, a + 13]
        .map(to_large_output)
        .into_iter()
        .collect()
}

fn seq(inputs: &[u32]) -> Vec<LargeOutput> {
    inputs.iter().flat_map(fmap).collect()
}

fn rayon(inputs: &[u32]) -> Vec<LargeOutput> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().flat_map(fmap).collect()
}

fn orx_parallel_default(inputs: &[u32]) -> Vec<LargeOutput> {
    inputs.into_par().flat_map(fmap).collect_vec()
}

fn orx_parallel(inputs: &[u32], num_threads: usize, chunk_size: usize) -> Vec<LargeOutput> {
    inputs
        .into_par()
        .chunk_size(ChunkSize::Exact(NonZeroUsize::new(chunk_size).unwrap()))
        .num_threads(num_threads)
        .flat_map(fmap)
        .collect_vec()
}

fn orx_parallel_x_default(inputs: &[u32]) -> Vec<LargeOutput> {
    inputs.into_par().flat_map(fmap).collect_x_vec()
}

fn orx_parallel_x(inputs: &[u32], num_threads: usize, chunk_size: usize) -> Vec<LargeOutput> {
    inputs
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .flat_map(fmap)
        .collect_x_vec()
}

fn validate(mut first: Vec<LargeOutput>, second: &[LargeOutput]) {
    first.sort();
    assert_eq!(first, second);
}

fn flat_map_large_output(c: &mut Criterion) {
    let treatments = vec![16_384, 65_536];

    let mut group = c.benchmark_group("flat_map_large_output");

    for n in &treatments {
        let input = inputs(*n);
        let mut expected = seq(&input);
        expected.sort();

        let params = [(32, n / 32)];

        group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
            b.iter(|| {
                let result = seq(black_box(&input));
                validate(result, &expected);
            })
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            b.iter(|| {
                let result = rayon(black_box(&input));
                validate(result, &expected);
            })
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-default", n), n, |b, _| {
            b.iter(|| {
                let result = orx_parallel_default(black_box(&input));
                validate(result, &expected);
            })
        });

        for (t, c) in params {
            let name = format!("orx-parallel-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(name, n), n, |b, _| {
                b.iter(|| {
                    let result = orx_parallel(black_box(&input), t, c);
                    validate(result, &expected);
                })
            });
        }

        group.bench_with_input(BenchmarkId::new("orx-parallel-x-default", n), n, |b, _| {
            b.iter(|| {
                let result = orx_parallel_x_default(black_box(&input));
                validate(result, &expected);
            })
        });

        for (t, c) in params {
            let name = format!("orx-parallel-x-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(name, n), n, |b, _| {
                b.iter(|| {
                    let result = orx_parallel_x(black_box(&input), t, c);
                    validate(result, &expected);
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, flat_map_large_output);
criterion_main!(benches);
