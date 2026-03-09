/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ff/iter/1024        time:   [918.37 ns 926.48 ns 935.42 ns]
xap_ff/xap/1024         time:   [907.83 ns 917.99 ns 928.18 ns]

xap_ff/iter/32768       time:   [114.23 µs 116.92 µs 119.97 µs]
xap_ff/xap/32768        time:   [120.63 µs 122.23 µs 124.04 µs]

xap_ff/iter/1048576     time:   [4.0164 ms 4.0487 ms 4.0829 ms]
xap_ff/xap/1048576      time:   [4.2115 ms 4.2395 ms 4.2687 ms]


COLLECT:
xap_ff/iter/1024        time:   [1.5479 µs 1.5638 µs 1.5812 µs]
xap_ff/xap/1024         time:   [2.0939 µs 2.1146 µs 2.1359 µs]

xap_ff/iter/32768       time:   [158.30 µs 159.47 µs 160.76 µs]
xap_ff/xap/32768        time:   [160.51 µs 162.08 µs 163.78 µs]

xap_ff/iter/1048576     time:   [5.7059 ms 5.7404 ms 5.7752 ms]
xap_ff/xap/1048576      time:   [5.5773 ms 5.6219 ms 5.6686 ms]

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
