/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ii/iter/1024        time:   [804.05 ns 811.27 ns 818.98 ns]
xap_ii/xap/1024         time:   [1.0405 µs 1.0503 µs 1.0606 µs]

xap_ii/iter/32768       time:   [56.637 µs 57.550 µs 58.470 µs]
xap_ii/xap/32768        time:   [129.85 µs 131.25 µs 132.82 µs]

xap_ii/iter/1048576     time:   [2.5822 ms 2.6058 ms 2.6299 ms]
xap_ii/xap/1048576      time:   [4.5383 ms 4.5844 ms 4.6345 ms]


COLLECT:
xap_ii/iter/1024        time:   [1.2865 µs 1.3013 µs 1.3169 µs]
xap_ii/xap/1024         time:   [1.7612 µs 1.7829 µs 1.8052 µs]

xap_ii/iter/32768       time:   [169.59 µs 171.53 µs 173.81 µs]
xap_ii/xap/32768        time:   [201.88 µs 205.46 µs 209.26 µs]

xap_ii/iter/1048576     time:   [6.6345 ms 6.7077 ms 6.7844 ms]
xap_ii/xap/1048576      time:   [7.5472 ms 7.7257 ms 7.9337 ms]

TODO: room for performance improvement

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, xap_variants::Id};
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

fn f1(i: u64) -> Option<u64> {
    match i.is_multiple_of(7) {
        true => None,
        false => Some(i + 3),
    }
}

fn f2(i: u64) -> Option<u64> {
    match i.is_multiple_of(3) {
        true => None,
        false => Some(2 * i),
    }
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter_map(f1).filter_map(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter_map(f1).filter_map(f2);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_ii");

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
