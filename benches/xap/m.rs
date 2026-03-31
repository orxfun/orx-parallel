/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_m/iter/1024         time:   [199.31 ns 200.63 ns 201.94 ns]
xap_m/xap/1024          time:   [202.65 ns 204.66 ns 206.76 ns]

xap_m/iter/32768        time:   [6.7413 µs 6.7875 µs 6.8344 µs]
xap_m/xap/32768         time:   [6.8722 µs 6.9204 µs 6.9726 µs]

xap_m/iter/1048576      time:   [290.64 µs 294.11 µs 297.73 µs]
xap_m/xap/1048576       time:   [281.04 µs 284.60 µs 288.52 µs]


COLLECT:
xap_m/iter/1024         time:   [210.80 ns 212.03 ns 213.29 ns]
xap_m/xap/1024          time:   [243.07 ns 245.32 ns 247.60 ns]

xap_m/iter/32768        time:   [6.8940 µs 6.9653 µs 7.0413 µs]
xap_m/xap/32768         time:   [7.0865 µs 7.1465 µs 7.2052 µs]

xap_m/iter/1048576      time:   [501.86 µs 507.82 µs 513.83 µs]
xap_m/xap/1048576       time:   [516.87 µs 521.67 µs 526.31 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, xap_variants::Id};
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
