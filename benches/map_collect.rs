use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const NUM_NUMBERS: usize = 4;
const NUM_VECTORS: usize = 4;
const LEN_VECTORS: usize = 4;
const SEED: u64 = 5426;
const FIB_UPPER_BOUND: u32 = 999;

#[derive(Debug, PartialEq, Eq)]
struct LargeOutput {
    name: String,
    numbers: [i64; NUM_NUMBERS],
    vectors: Vec<Vec<i64>>,
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

    let mut numbers = [0i64; NUM_NUMBERS];
    for (i, x) in numbers.iter_mut().enumerate() {
        *x = match (idx * 7 + i) % 3 {
            0 => idx as i64 + i as i64,
            _ => idx as i64 - i as i64,
        };
    }

    let mut vectors = vec![];
    for i in 0..NUM_VECTORS {
        let mut vec = vec![];
        for j in 0..(idx % LEN_VECTORS) {
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

fn orx_parallel_vec(inputs: &[usize]) -> Vec<LargeOutput> {
    inputs.into_par().map(to_large_output).collect_vec()
}

fn orx_parallel_split_vec(inputs: &[usize]) -> SplitVec<LargeOutput> {
    inputs.into_par().map(to_large_output).collect()
}

fn orx_parallel(inputs: &[usize], num_threads: usize, chunk_size: usize) -> SplitVec<LargeOutput> {
    inputs
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .map(to_large_output)
        .collect()
}

fn map_collect(c: &mut Criterion) {
    let treatments = [65_536, 65_536 * 4];
    let _params = [(1, 1), (32, 1024)];
    let params = [];

    let mut group = c.benchmark_group("map_collect");

    for n in &treatments {
        let input = inputs(*n);
        let expected = seq(&input);

        group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
            assert_eq!(&expected, &seq(&input));
            b.iter(|| seq(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            assert_eq!(&expected, &rayon(&input));
            b.iter(|| rayon(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-split-vec", n), n, |b, _| {
            assert_eq!(&expected, &orx_parallel_split_vec(&input));
            b.iter(|| orx_parallel_split_vec(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-vec", n), n, |b, _| {
            assert_eq!(&expected, &orx_parallel_vec(&input));
            b.iter(|| orx_parallel_vec(black_box(&input)))
        });

        for (t, c) in params {
            let name = format!("orx-parallel-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(name, n), n, |b, _| {
                assert_eq!(&expected, &orx_parallel(&input, t, c));
                b.iter(|| orx_parallel(black_box(&input), t, c))
            });
        }
    }

    group.finish();
}

criterion_group!(benches, map_collect);
criterion_main!(benches);
