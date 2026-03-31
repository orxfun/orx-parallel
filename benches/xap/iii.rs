/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_iii/iter/1024       time:   [1.0427 µs 1.0517 µs 1.0614 µs]
xap_iii/xap/1024        time:   [1.5707 µs 1.5984 µs 1.6280 µs]

xap_iii/iter/32768      time:   [72.918 µs 74.710 µs 76.277 µs]
xap_iii/xap/32768       time:   [78.941 µs 79.989 µs 81.193 µs]

xap_iii/iter/1048576    time:   [2.5374 ms 2.5682 ms 2.6006 ms]
xap_iii/xap/1048576     time:   [3.1935 ms 3.2185 ms 3.2435 ms]


COLLECT:
xap_iii/iter/1024       time:   [1.9682 µs 2.0051 µs 2.0412 µs]
xap_iii/xap/1024        time:   [2.7314 µs 2.7785 µs 2.8304 µs]

xap_iii/iter/32768      time:   [187.48 µs 190.12 µs 193.27 µs]
xap_iii/xap/32768       time:   [189.11 µs 192.10 µs 195.28 µs]

xap_iii/iter/1048576    time:   [7.2220 ms 7.2930 ms 7.3674 ms]
xap_iii/xap/1048576     time:   [7.6169 ms 7.7408 ms 7.8721 ms]

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

fn f3(i: u64) -> Option<u64> {
    match (i + 5).is_multiple_of(4) {
        true => None,
        false => Some(3 * i + 1),
    }
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs
        .iter()
        .copied()
        .filter_map(f1)
        .filter_map(f2)
        .filter_map(f3);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter_map(f1).filter_map(f2).filter_map(f3);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_iii");

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
