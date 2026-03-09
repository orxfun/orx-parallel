/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ll/iter/1024        time:   [3.7371 µs 3.7553 µs 3.7739 µs]
xap_ll/xap/1024         time:   [10.224 µs 10.290 µs 10.356 µs]

xap_ll/iter/32768       time:   [120.36 µs 120.99 µs 121.62 µs]
xap_ll/xap/32768        time:   [332.57 µs 334.81 µs 337.06 µs]

xap_ll/iter/1048576     time:   [3.8710 ms 3.8878 ms 3.9046 ms]
xap_ll/xap/1048576      time:   [10.678 ms 10.741 ms 10.807 ms]


COLLECT:
xap_ll/iter/1024        time:   [4.1939 µs 4.2358 µs 4.2788 µs]
xap_ll/xap/1024         time:   [37.790 µs 38.423 µs 39.039 µs]

xap_ll/iter/32768       time:   [192.32 µs 194.13 µs 196.18 µs]
xap_ll/xap/32768        time:   [1.1706 ms 1.1917 ms 1.2127 ms]

xap_ll/iter/1048576     time:   [65.585 ms 66.152 ms 66.795 ms]
xap_ll/xap/1048576      time:   [95.022 ms 96.462 ms 98.320 ms]

(!) significant difference

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
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f1(i: u64) -> Vec<u64> {
    vec![i + 1, i * 2, i + 5, i + 4, i, i.saturating_sub(3), 7 * i]
}

fn f2(i: u64) -> Vec<u64> {
    vec![i * 2 + 1, i, i.saturating_sub(7)]
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1).flat_map(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).flat_map(f2);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15 /*, 1 << 20*/];

    let mut group = c.benchmark_group("xap_ll");

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
