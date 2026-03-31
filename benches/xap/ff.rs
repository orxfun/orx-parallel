/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ff/iter/1024        time:   [893.92 ns 908.70 ns 923.11 ns]
xap_ff/xap/1024         time:   [829.30 ns 837.74 ns 846.69 ns]

xap_ff/iter/32768       time:   [103.16 µs 104.07 µs 105.01 µs]
xap_ff/xap/32768        time:   [112.06 µs 113.29 µs 114.62 µs]

xap_ff/iter/1048576     time:   [3.9376 ms 3.9759 ms 4.0158 ms]
xap_ff/xap/1048576      time:   [4.0581 ms 4.0876 ms 4.1184 ms]


COLLECT:
xap_ff/iter/1024        time:   [1.9182 µs 1.9469 µs 1.9774 µs]
xap_ff/xap/1024         time:   [2.1158 µs 2.1418 µs 2.1695 µs]

xap_ff/iter/32768       time:   [184.96 µs 187.84 µs 190.94 µs]
xap_ff/xap/32768        time:   [179.58 µs 182.34 µs 185.05 µs]

xap_ff/iter/1048576     time:   [7.6464 ms 7.8468 ms 8.0493 ms]
xap_ff/xap/1048576      time:   [6.2378 ms 6.3169 ms 6.3994 ms]

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

fn f1(i: &u64) -> bool {
    !i.is_multiple_of(3)
}

fn f2(i: &u64) -> bool {
    !(i + 7).is_multiple_of(11)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter(f1).filter(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter(f1).filter(f2);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_ff");

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
