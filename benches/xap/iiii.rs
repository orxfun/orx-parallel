/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_iiii/iter/1024      time:   [1.0887 µs 1.0942 µs 1.1000 µs]
xap_iiii/xap/1024       time:   [1.0350 µs 1.0423 µs 1.0495 µs]

xap_iiii/iter/32768     time:   [119.88 µs 120.86 µs 121.93 µs]
xap_iiii/xap/32768      time:   [130.87 µs 131.67 µs 132.49 µs]

xap_iiii/iter/1048576   time:   [5.1265 ms 5.1558 ms 5.1855 ms]
xap_iiii/xap/1048576    time:   [4.5817 ms 4.6149 ms 4.6505 ms]


COLLECT:
xap_iiii/iter/1024      time:   [1.7605 µs 1.7763 µs 1.7923 µs]
xap_iiii/xap/1024       time:   [1.7478 µs 1.7627 µs 1.7778 µs]

xap_iiii/iter/32768     time:   [180.77 µs 182.33 µs 184.12 µs]
xap_iiii/xap/32768      time:   [148.89 µs 150.30 µs 151.71 µs]

xap_iiii/iter/1048576   time:   [6.7895 ms 6.8296 ms 6.8707 ms]
xap_iiii/xap/1048576    time:   [5.7793 ms 5.8156 ms 5.8528 ms]

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

fn f3(i: u64) -> Option<u64> {
    match (i + 5).is_multiple_of(4) {
        true => None,
        false => Some(3 * i + 1),
    }
}

fn f4(i: u64) -> Option<u64> {
    match i.is_multiple_of(11) {
        true => None,
        false => Some(2 * i + 5),
    }
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs
        .iter()
        .copied()
        .filter_map(f1)
        .filter_map(f2)
        .filter_map(f3)
        .filter_map(f4);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new()
        .filter_map(f1)
        .filter_map(f2)
        .filter_map(f3)
        .filter_map(f4);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_iiii");

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
