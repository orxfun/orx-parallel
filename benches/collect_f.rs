use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use orx_split_vec::{PinnedVec, SplitVec};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

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

fn filter(output: &&Output) -> bool {
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

fn inputs(len: usize) -> Vec<Output> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.gen_range(0..FIB_UPPER_BOUND) as usize)
        .map(|x| to_output(&x))
        .collect()
}

fn seq(inputs: &[Output]) -> Vec<&Output> {
    inputs.iter().filter(filter).collect()
}

fn rayon(inputs: &[Output]) -> Vec<&Output> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().filter(filter).collect()
}

fn orx_sorted_vec(inputs: &[Output]) -> Vec<&Output> {
    inputs
        .into_par()
        .filter(filter)
        .collect_ordering(CollectOrdering::SortWithHeap)
        .collect()
}

fn orx_arbitrary_vec(inputs: &[Output]) -> Vec<&Output> {
    inputs
        .into_par()
        .filter(filter)
        .collect_ordering(CollectOrdering::Arbitrary)
        .collect()
}

fn orx_sorted_split_vec(inputs: &[Output]) -> SplitVec<&Output> {
    inputs
        .into_par()
        .filter(filter)
        .collect_ordering(CollectOrdering::SortWithHeap)
        .collect()
}

fn orx_arbitrary_split_vec(inputs: &[Output]) -> SplitVec<&Output> {
    inputs
        .into_par()
        .filter(filter)
        .collect_ordering(CollectOrdering::Arbitrary)
        .collect()
}

fn map_collect(c: &mut Criterion) {
    let treatments = [65_536, 65_536 * 4];

    let mut group = c.benchmark_group("map_collect");

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

        group.bench_with_input(BenchmarkId::new("orx-sorted-vec", n), n, |b, _| {
            assert_eq!(&expected, &orx_sorted_vec(&input));
            b.iter(|| orx_sorted_vec(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx-sorted-split-vec", n), n, |b, _| {
            assert_eq!(&expected, &orx_sorted_split_vec(&input));
            b.iter(|| orx_sorted_split_vec(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx-arbitrary-vec", n), n, |b, _| {
            let mut arbitrary = orx_arbitrary_vec(&input);
            arbitrary.sort();
            assert_eq!(&sorted, &arbitrary);
            b.iter(|| orx_arbitrary_vec(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx-arbitrary-split-vec", n), n, |b, _| {
            let mut arbitrary = orx_arbitrary_split_vec(&input);
            arbitrary.sort();
            assert_eq!(&sorted, &arbitrary);
            b.iter(|| orx_arbitrary_split_vec(black_box(&input)))
        });
    }

    group.finish();
}

criterion_group!(benches, map_collect);
criterion_main!(benches);
