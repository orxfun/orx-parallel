/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_iiii/iter/1024      time:   [1.3731 µs 1.3859 µs 1.3996 µs]
xap_iiii/xap/1024       time:   [1.1241 µs 1.1394 µs 1.1551 µs]

xap_iiii/iter/32768     time:   [187.17 µs 189.54 µs 192.09 µs]
xap_iiii/xap/32768      time:   [120.21 µs 121.81 µs 123.65 µs]

xap_iiii/iter/1048576   time:   [6.5249 ms 6.5925 ms 6.6643 ms]
xap_iiii/xap/1048576    time:   [5.4545 ms 5.5254 ms 5.6033 ms]


COLLECT:
xap_iiii/iter/1024      time:   [2.2998 µs 2.3284 µs 2.3597 µs]
xap_iiii/xap/1024       time:   [2.3037 µs 2.3343 µs 2.3647 µs]

xap_iiii/iter/32768     time:   [230.91 µs 233.56 µs 236.35 µs]
xap_iiii/xap/32768      time:   [191.90 µs 194.63 µs 197.45 µs]

xap_iiii/iter/1048576   time:   [8.2246 ms 8.3142 ms 8.4106 ms]
xap_iiii/xap/1048576    time:   [6.9831 ms 7.0610 ms 7.1440 ms]

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
