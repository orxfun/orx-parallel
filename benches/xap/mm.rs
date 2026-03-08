/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_mm/iter/1024        time:   [625.36 ns 632.03 ns 638.94 ns]
xap_mm/xap/1024         time:   [595.67 ns 599.25 ns 603.12 ns]

xap_mm/iter/32768       time:   [18.842 µs 18.972 µs 19.103 µs]
xap_mm/xap/32768        time:   [19.029 µs 19.157 µs 19.291 µs]

xap_mm/iter/1048576     time:   [656.82 µs 662.92 µs 669.22 µs]
xap_mm/xap/1048576      time:   [644.57 µs 650.41 µs 656.65 µs]


COLLECT:
xap_mm/iter/1024        time:   [531.58 ns 535.03 ns 538.66 ns]
xap_mm/xap/1024         time:   [592.78 ns 597.47 ns 602.37 ns]

xap_mm/iter/32768       time:   [17.179 µs 17.323 µs 17.467 µs]
xap_mm/xap/32768        time:   [17.714 µs 17.883 µs 18.071 µs]

xap_mm/iter/1048576     time:   [690.13 µs 697.96 µs 705.61 µs]
xap_mm/xap/1048576      time:   [703.92 µs 721.28 µs 739.81 µs]

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

fn f1(i: u64) -> u64 {
    2 * i + 1
}

fn f2(i: u64) -> u64 {
    (7 * i).saturating_sub(71)
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

    let mut group = c.benchmark_group("xap_mm");

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
