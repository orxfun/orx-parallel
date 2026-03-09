/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_iii/iter/1024       time:   [780.92 ns 785.57 ns 790.86 ns]
xap_iii/xap/1024        time:   [796.96 ns 802.17 ns 807.29 ns]

xap_iii/iter/32768      time:   [46.491 µs 46.968 µs 47.432 µs]
xap_iii/xap/32768       time:   [96.832 µs 97.809 µs 98.837 µs]

xap_iii/iter/1048576    time:   [2.1013 ms 2.1204 ms 2.1395 ms]
xap_iii/xap/1048576     time:   [4.3454 ms 4.3667 ms 4.3881 ms]


COLLECT:
xap_iii/iter/1024       time:   [1.5568 µs 1.5674 µs 1.5787 µs]
xap_iii/xap/1024        time:   [1.5886 µs 1.6002 µs 1.6118 µs]

xap_iii/iter/32768      time:   [153.17 µs 154.37 µs 155.61 µs]
xap_iii/xap/32768       time:   [127.73 µs 128.83 µs 130.01 µs]

xap_iii/iter/1048576    time:   [5.8130 ms 5.8440 ms 5.8766 ms]
xap_iii/xap/1048576     time:   [5.2404 ms 5.2918 ms 5.3447 ms]

(!) SUM has a significant difference

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
