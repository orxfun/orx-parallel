use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use orx_parallel::*;
use orx_split_vec::SplitVec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const TEST_LARGE_OUTPUT: bool = false;

const LARGE_OUTPUT_LEN: usize = match TEST_LARGE_OUTPUT {
    true => 64,
    false => 0,
};
const SEED: u64 = 9562;
const FIB_UPPER_BOUND: u32 = 201;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Output {
    name: String,
    numbers: [i64; LARGE_OUTPUT_LEN],
}

fn map(idx: &usize) -> Output {
    let idx = *idx;
    let prefix = match idx % 7 {
        0 => "zero-",
        3 => "three-",
        _ => "sth-",
    };
    let fib = fibonacci(&(idx as u32));
    let name = format!("{}-fib-{}", prefix, fib);

    let mut numbers = [0i64; LARGE_OUTPUT_LEN];
    for (i, x) in numbers.iter_mut().enumerate() {
        *x = match (idx * 7 + i) % 3 {
            0 => idx as i64 + i as i64,
            _ => idx as i64 - i as i64,
        };
    }

    Output { name, numbers }
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
        .map(|_| rng.random_range(0..FIB_UPPER_BOUND) as usize)
        .collect()
}

fn seq(inputs: &[usize]) -> Vec<Output> {
    inputs.iter().chain(inputs).chain(inputs).map(map).collect()
}

fn rayon(inputs: &[usize]) -> Vec<Output> {
    use rayon::iter::ParallelIterator;
    inputs
        .into_par_iter()
        .chain(inputs)
        .chain(inputs)
        .map(map)
        .collect()
}

fn orx_into_vec(inputs: &[usize]) -> Vec<Output> {
    inputs
        .into_par()
        .chain(inputs)
        .chain(inputs)
        .map(map)
        .collect()
}

fn orx_into_split_vec(inputs: &[usize]) -> SplitVec<Output> {
    inputs
        .into_par()
        .chain(inputs)
        .chain(inputs)
        .map(map)
        .collect()
}

fn run(c: &mut Criterion) {
    let treatments = [65_536 * 2];

    let mut group = c.benchmark_group("chain3_collect_map");

    for n in &treatments {
        let input = inputs(*n);
        let expected = seq(&input);
        let mut sorted = seq(&input);
        sorted.sort();

        group.bench_with_input(BenchmarkId::new("seq-into-vec", n), n, |b, _| {
            assert_eq!(&expected, &seq(&input));
            b.iter(|| seq(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("rayon-into-vec", n), n, |b, _| {
            assert_eq!(&expected, &rayon(&input));
            b.iter(|| rayon(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx-into-vec", n), n, |b, _| {
            assert_eq!(&expected, &orx_into_vec(&input));
            b.iter(|| orx_into_vec(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx-into-split-vec", n), n, |b, _| {
            assert_eq!(&expected, &orx_into_split_vec(&input));
            b.iter(|| orx_into_split_vec(black_box(&input)))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
