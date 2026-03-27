use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::IntoConcurrentIter;
use orx_parallel::infallible::par;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn seq(input: &[u64]) -> Option<u64> {
    input.iter().next().copied()
}

fn orx(input: &[u64]) -> Option<u64> {
    par(input.into_con_iter()).first().copied()
}

fn rayon(input: &[u64]) -> Option<u64> {
    input.into_par_iter().find_first(|_| true).copied()
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("first_id");

    for n in len {
        let input = inputs(n);
        let expected = seq(&input);

        group.bench_with_input(BenchmarkId::new("seq", n), &n, |b, _| {
            assert_eq!(&expected, &seq(&input));
            b.iter(|| seq(&input))
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), &n, |b, _| {
            assert_eq!(&expected, &rayon(&input));
            b.iter(|| rayon(&input))
        });

        group.bench_with_input(BenchmarkId::new("orx", n), &n, |b, _| {
            assert_eq!(&expected, &orx(&input));
            b.iter(|| orx(&input))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
