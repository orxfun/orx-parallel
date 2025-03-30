use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
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

fn to_output(idx: &usize) -> Output {
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

fn filter(a: &Output) -> bool {
    !a.name.ends_with('1')
}

fn reduce(a: Output, b: Output) -> Output {
    match a.name < b.name {
        true => a,
        false => b,
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

fn seq(inputs: &[usize]) -> Option<Output> {
    inputs.iter().map(to_output).filter(filter).reduce(reduce)
}

fn rayon(inputs: &[usize]) -> Option<Output> {
    use rayon::iter::ParallelIterator;
    inputs
        .into_par_iter()
        .map(to_output)
        .filter(filter)
        .reduce_with(reduce)
}

fn orx_sequential(inputs: &[usize]) -> Option<Output> {
    inputs
        .into_par()
        .map(to_output)
        .filter(filter)
        .num_threads(1)
        .reduce(reduce)
}

fn orx(inputs: &[usize]) -> Option<Output> {
    inputs
        .into_par()
        .map(to_output)
        .filter(filter)
        .reduce(reduce)
}

fn run(c: &mut Criterion) {
    let treatments = [65_536, 65_536 * 4];

    let mut group = c.benchmark_group("reduce_map_filter");

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

        group.bench_with_input(BenchmarkId::new("orx-sequential", n), n, |b, _| {
            assert_eq!(&expected, &orx_sequential(&input));
            b.iter(|| orx_sequential(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx", n), n, |b, _| {
            assert_eq!(&expected, &orx(&input));
            b.iter(|| orx(black_box(&input)))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
