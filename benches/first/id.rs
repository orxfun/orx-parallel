/*

first_id/seq/1024       time:   [459.73 ps 465.10 ps 470.46 ps]
first_id/rayon/1024     time:   [2.4319 ms 2.4850 ms 2.5387 ms]
first_id/orx/1024       time:   [1.8087 ms 1.8554 ms 1.9051 ms]

first_id/seq/32768      time:   [501.31 ps 515.28 ps 531.02 ps]
first_id/rayon/32768    time:   [3.0993 ms 3.2035 ms 3.3115 ms]
first_id/orx/32768      time:   [3.2926 ms 3.4211 ms 3.5529 ms]

first_id/seq/1048576    time:   [551.79 ps 566.80 ps 580.32 ps]
first_id/rayon/1048576  time:   [2.6398 ms 2.7137 ms 2.7881 ms]
first_id/orx/1048576    time:   [1.8319 ms 1.8669 ms 1.9032 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
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
    input.into_par().first().copied()
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
