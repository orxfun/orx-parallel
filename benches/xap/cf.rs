/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_cf/iter/1024        time:   [638.02 ns 642.61 ns 647.44 ns]
xap_cf/xap/1024         time:   [694.96 ns 706.95 ns 719.65 ns]

xap_cf/iter/32768       time:   [63.537 µs 64.597 µs 65.679 µs]
xap_cf/xap/32768        time:   [63.717 µs 64.404 µs 65.106 µs]

xap_cf/iter/1048576     time:   [3.3602 ms 3.3812 ms 3.4022 ms]
xap_cf/xap/1048576      time:   [3.4023 ms 3.4355 ms 3.4717 ms]


COLLECT:
xap_cf/iter/1024        time:   [1.0456 µs 1.0519 µs 1.0581 µs]
xap_cf/xap/1024         time:   [1.1446 µs 1.1504 µs 1.1561 µs]

xap_cf/iter/32768       time:   [116.61 µs 117.38 µs 118.20 µs]
xap_cf/xap/32768        time:   [120.35 µs 120.95 µs 121.59 µs]

xap_cf/iter/1048576     time:   [4.3939 ms 4.4169 ms 4.4403 ms]
xap_cf/xap/1048576      time:   [4.2362 ms 4.2640 ms 4.2931 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, fun::FnCopied, xap_variants::Id};
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

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter(f1);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().mapped(FnCopied::new()).filter(f1);
    E::out(inputs.iter().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_cf");

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
