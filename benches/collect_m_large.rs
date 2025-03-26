use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use orx_split_vec::SplitVec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const LARGE_OUTPUT_LEN: usize = 64;
const SEED: u64 = 5426;
const FIB_UPPER_BOUND: u32 = 999;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Output {
    name: String,
    numbers: [i64; LARGE_OUTPUT_LEN],
}

fn to_large_output(idx: &usize) -> Output {
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
        .map(|_| rng.gen_range(0..FIB_UPPER_BOUND) as usize)
        .collect()
}

fn seq(inputs: &[usize]) -> Vec<Output> {
    inputs.iter().map(to_large_output).collect()
}

fn rayon(inputs: &[usize]) -> Vec<Output> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().map(to_large_output).collect()
}

fn orx_parallel_vec(inputs: &[usize]) -> Vec<Output> {
    inputs.into_par().map(to_large_output).collect()
}

fn orx_parallel_vec_arbitrary(inputs: &[usize]) -> Vec<Output> {
    inputs
        .into_par()
        .map(to_large_output)
        .collect_ordering(CollectOrdering::Arbitrary)
        .collect()
}

fn orx_parallel_split_vec(inputs: &[usize]) -> SplitVec<Output> {
    inputs.into_par().map(to_large_output).collect()
}

fn orx_parallel(inputs: &[usize], num_threads: usize, chunk_size: usize) -> SplitVec<Output> {
    inputs
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .map(to_large_output)
        .collect()
}

fn map_collect(c: &mut Criterion) {
    let treatments = [65_536, 65_536 * 4];
    let params = [(8, 64), (8, 1024)];

    let mut group = c.benchmark_group("map_collect");

    for n in &treatments {
        let input = inputs(*n);
        let expected = seq(&input);
        let mut sorted_expected = seq(&input);
        sorted_expected.sort();

        group.bench_with_input(BenchmarkId::new("seq-into-vec", n), n, |b, _| {
            assert_eq!(&expected, &seq(&input));
            b.iter(|| seq(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("rayon-into-vec", n), n, |b, _| {
            assert_eq!(&expected, &rayon(&input));
            b.iter(|| rayon(black_box(&input)))
        });

        group.bench_with_input(
            BenchmarkId::new("orx-parallel-into-split-vec", n),
            n,
            |b, _| {
                assert_eq!(&expected, &orx_parallel_split_vec(&input));
                b.iter(|| orx_parallel_split_vec(black_box(&input)))
            },
        );

        group.bench_with_input(BenchmarkId::new("orx-parallel-into-vec", n), n, |b, _| {
            assert_eq!(&expected, &orx_parallel_vec(&input));
            b.iter(|| orx_parallel_vec(black_box(&input)))
        });

        group.bench_with_input(
            BenchmarkId::new("orx-parallel-into-vec-arbitrary", n),
            n,
            |b, _| {
                let mut output = orx_parallel_vec_arbitrary(&input);
                output.sort();
                assert_eq!(&sorted_expected, &output);
                b.iter(|| orx_parallel_vec(black_box(&input)))
            },
        );

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
