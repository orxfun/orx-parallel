/*
The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_map_map/iter/1024   time:   [204.57 ns 206.40 ns 208.30 ns]
xap_map_map/xap/1024    time:   [203.04 ns 204.81 ns 206.68 ns]

xap_map_map/iter/32768  time:   [6.0384 µs 6.0882 µs 6.1408 µs]
xap_map_map/xap/32768   time:   [5.9940 µs 6.0283 µs 6.0655 µs]

xap_map_map/iter/1048576time:   [253.07 µs 255.53 µs 258.01 µs]
xap_map_map/xap/1048576 time:   [243.62 µs 246.05 µs 248.60 µs]


COLLECT:
xap_map_map/iter/1024   time:   [196.83 ns 197.76 ns 198.74 ns]
xap_map_map/xap/1024    time:   [233.82 ns 235.73 ns 237.68 ns]

xap_map_map/iter/32768  time:   [6.6075 µs 6.6655 µs 6.7219 µs]
xap_map_map/xap/32768   time:   [6.5634 µs 6.6086 µs 6.6552 µs]

xap_map_map/iter/1048576time:   [411.03 µs 413.90 µs 416.97 µs]
xap_map_map/xap/1048576 time:   [440.65 µs 442.98 µs 445.44 µs]

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

fn f1(i: u64) -> u64 {
    2 * i + 1
}

fn f2(i: u64) -> u64 {
    i / 2 + 17
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().map(f1).map(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().map(f1).map(f2);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_m_m");

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
