use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 9562;
const FIB_UPPER_BOUND: u32 = 201;

fn to_output(idx: &usize) -> u32 {
    let idx = *idx;
    fibonacci(&(idx as u32))
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

fn inputs(len: usize) -> Vec<u32> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.random_range(0..FIB_UPPER_BOUND) as usize)
        .map(|x| to_output(&x))
        .collect()
}

fn seq(inputs: &[u32]) -> u32 {
    inputs.into_iter().sum()
}

fn rayon(inputs: &[u32]) -> u32 {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().sum()
}

fn orx(inputs: &[u32]) -> u32 {
    inputs.into_par().sum()
}

fn run(c: &mut Criterion) {
    let treatments = [65_536, 65_536 * 4];

    let mut group = c.benchmark_group("sum");

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

        group.bench_with_input(BenchmarkId::new("orx", n), n, |b, _| {
            assert_eq!(&expected, &orx(&input));
            b.iter(|| orx(black_box(&input)))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
