/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ff/iter/1024        time:   [638.83 ns 643.00 ns 647.65 ns]
xap_ff/xap/1024         time:   [761.30 ns 766.44 ns 771.75 ns]

xap_ff/iter/32768       time:   [108.09 µs 108.76 µs 109.41 µs]
xap_ff/xap/32768        time:   [103.35 µs 104.29 µs 105.31 µs]

xap_ff/iter/1048576     time:   [4.2004 ms 4.2433 ms 4.2877 ms]
xap_ff/xap/1048576      time:   [4.3339 ms 4.3972 ms 4.4637 ms]


COLLECT:
xap_ff/iter/1024        time:   [1.4958 µs 1.5082 µs 1.5213 µs]
xap_ff/xap/1024         time:   [1.9230 µs 1.9402 µs 1.9595 µs]

xap_ff/iter/32768       time:   [132.85 µs 134.27 µs 135.80 µs]
xap_ff/xap/32768        time:   [154.63 µs 156.01 µs 157.51 µs]

xap_ff/iter/1048576     time:   [5.1437 ms 5.1915 ms 5.2418 ms]
xap_ff/xap/1048576      time:   [5.8194 ms 5.8701 ms 5.9222 ms]

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
