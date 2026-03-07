/*
The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_map/iter/1024       time:   [207.92 ns 210.84 ns 213.72 ns]
xap_map/xap/1024        time:   [202.03 ns 205.27 ns 208.54 ns]

xap_map/iter/32768      time:   [6.4050 µs 6.4663 µs 6.5309 µs]
xap_map/xap/32768       time:   [6.4592 µs 6.5383 µs 6.6250 µs]

xap_map/iter/1048576    time:   [249.34 µs 252.93 µs 256.45 µs]
xap_map/xap/1048576     time:   [231.12 µs 233.48 µs 235.76 µs]


COLLECT:
xap_map/iter/1024       time:   [227.21 ns 232.67 ns 238.16 ns]
xap_map/xap/1024        time:   [247.15 ns 250.31 ns 253.65 ns]

xap_map/iter/32768      time:   [6.8360 µs 6.8860 µs 6.9357 µs]
xap_map/xap/32768       time:   [7.2622 µs 7.3775 µs 7.4829 µs]

xap_map/iter/1048576    time:   [453.18 µs 457.17 µs 461.33 µs]
xap_map/xap/1048576     time:   [482.77 µs 491.98 µs 502.11 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = Sum;

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
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f(i: u64) -> u64 {
    2 * i + 1
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().map(f);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().map(f);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_m");

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
