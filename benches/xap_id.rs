/*
The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.
The overhead is negligible, zero-cost abstraction seems to work.


SUM:
xap_id/iter/1024        time:   [94.476 ns 95.187 ns 95.894 ns]
xap_id/xap/1024         time:   [97.750 ns 98.687 ns 99.649 ns]

xap_id/iter/32768       time:   [2.7868 µs 2.8128 µs 2.8413 µs]
xap_id/xap/32768        time:   [2.8619 µs 2.8795 µs 2.8968 µs]

xap_id/iter/1048576     time:   [149.47 µs 150.61 µs 151.75 µs]
xap_id/xap/1048576      time:   [151.68 µs 152.73 µs 153.78 µs]


COLLECT:
xap_id/iter/1024        time:   [102.03 ns 102.91 ns 103.82 ns]
xap_id/xap/1024         time:   [123.55 ns 124.31 ns 125.09 ns]

xap_id/iter/32768       time:   [4.9836 µs 5.0239 µs 5.0704 µs]
xap_id/xap/32768        time:   [4.9354 µs 4.9677 µs 5.0016 µs]

xap_id/iter/1048576     time:   [344.57 µs 348.00 µs 351.63 µs]
xap_id/xap/1048576      time:   [354.44 µs 357.16 µs 360.05 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = Collect;

trait Exp {
    type Out;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out;
}

pub struct Sum;
impl Exp for Sum {
    type Out = u64;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        i.sum()
    }
}

pub struct Collect;
impl Exp for Collect {
    type Out = Vec<u64>;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        i.collect()
    }
}

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 9562;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    E::out(inputs.iter().copied())
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new();
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_id");

    for n in len {
        let input = inputs(n);
        let expected = iter::<Output>(&input);

        group.bench_with_input(BenchmarkId::new("iter", n), &n, |b, _| {
            assert_eq!(&expected, &iter::<Output>(&input));
            b.iter(|| iter::<Output>(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("xap", n), &n, |b, _| {
            assert_eq!(&expected, &xap::<Output>(&input));
            b.iter(|| xap::<Output>(black_box(&input)))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
