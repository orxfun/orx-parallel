use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use orx_parallel::*;
use orx_split_vec::SplitVec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

const TEST_LARGE_OUTPUT: bool = false;

const LARGE_OUTPUT_LEN: usize = match TEST_LARGE_OUTPUT {
    true => 64,
    false => 0,
};
const SEED: u64 = 5426;
const FIB_UPPER_BOUND: u32 = 201;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Output {
    name: String,
    numbers: [i64; LARGE_OUTPUT_LEN],
}

fn map(idx: usize) -> Output {
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

fn filter(output: &Output) -> bool {
    let last_char = output.name.chars().last().unwrap();
    let last_digit: u32 = last_char.to_string().parse().unwrap();
    last_digit < 4
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

fn seq(input: &mut Vec<usize>) -> Vec<Output> {
    input.drain(..).map(map).filter(filter).collect()
}

fn rayon(input: &mut Vec<usize>) -> Vec<Output> {
    use rayon::iter::ParallelIterator;
    rayon::iter::ParallelDrainRange::par_drain(input, ..)
        .map(map)
        .filter(filter)
        .collect()
}

fn orx_into_vec(input: &mut Vec<usize>) -> Vec<Output> {
    input.par_drain(..).map(map).filter(filter).collect()
}

fn orx_into_split_vec(input: &mut Vec<usize>) -> SplitVec<Output> {
    input.par_drain(..).map(map).filter(filter).collect()
}

fn run(c: &mut Criterion) {
    let treatments = [65_536 * 2];

    let mut group = c.benchmark_group("drain_vec_collect_map_filter");

    for n in &treatments {
        let input = inputs(*n);
        let expected = seq(&mut input.clone());

        group.bench_with_input(BenchmarkId::new("seq-into-vec", n), n, |b, _| {
            assert_eq!(&expected, &seq(&mut input.clone()));
            b.iter(|| seq(black_box(&mut input.clone())))
        });

        group.bench_with_input(BenchmarkId::new("rayon-into-vec", n), n, |b, _| {
            assert_eq!(&expected, &rayon(&mut input.clone()));
            b.iter(|| rayon(black_box(&mut input.clone())))
        });

        group.bench_with_input(BenchmarkId::new("orx-into-vec", n), n, |b, _| {
            assert_eq!(&expected, &orx_into_vec(&mut input.clone()));
            b.iter(|| orx_into_vec(black_box(&mut input.clone())))
        });

        group.bench_with_input(BenchmarkId::new("orx-into-split-vec", n), n, |b, _| {
            assert_eq!(&expected, &orx_into_split_vec(&mut input.clone()));
            b.iter(|| orx_into_split_vec(black_box(&mut input.clone())))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
