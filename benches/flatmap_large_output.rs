use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use orx_split_vec::{Recursive, SplitVec};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 89774;
const FIB_UPPER_BOUND: u32 = 30;

#[derive(PartialEq, Eq, Debug, Clone)]
struct LargeOutput {
    idx: usize,
    name: String,
    numbers: [i64; 64],
    vectors: Vec<Vec<i64>>,
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

fn orx_parallel_split_vec(inputs: &[u32]) -> SplitVec<LargeOutput> {
    inputs.into_par().flat_map(fmap).collect()
}

fn orx_parallel_split_rec(inputs: &[u32]) -> SplitVec<LargeOutput, Recursive> {
    inputs.into_par().flat_map(fmap).collect_x()
}

fn orx_parallel_vec(inputs: &[u32]) -> Vec<LargeOutput> {
    inputs.into_par().flat_map(fmap).collect_vec()
}

fn flat_map_large_output(c: &mut Criterion) {
    let treatments = vec![16_384, 65_536];

    let mut group = c.benchmark_group("flat_map_large_output");

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
    }

    group.finish();
}

criterion_group!(benches, flat_map_large_output);
criterion_main!(benches);
