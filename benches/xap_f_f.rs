/*
The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_f/iter/1024         time:   [490.72 ns 493.51 ns 496.29 ns]
xap_f/xap/1024          time:   [496.50 ns 499.57 ns 502.81 ns]

xap_f/iter/32768        time:   [19.216 µs 19.914 µs 20.673 µs]
xap_f/xap/32768         time:   [20.652 µs 21.574 µs 22.585 µs]

xap_f/iter/1048576      time:   [2.2176 ms 2.2271 ms 2.2367 ms]
xap_f/xap/1048576       time:   [2.2115 ms 2.2222 ms 2.2333 ms]


COLLECT:
xap_f_f/iter/1024       time:   [824.13 ns 828.24 ns 832.45 ns]
xap_f_f/xap/1024        time:   [919.75 ns 924.18 ns 928.43 ns]

xap_f_f/iter/32768      time:   [60.553 µs 60.890 µs 61.254 µs]
xap_f_f/xap/32768       time:   [57.590 µs 57.969 µs 58.388 µs]

xap_f_f/iter/1048576    time:   [2.5854 ms 2.6053 ms 2.6268 ms]
xap_f_f/xap/1048576     time:   [2.6348 ms 2.6629 ms 2.6923 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
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
    !(2 * i + 1).is_multiple_of(5)
}

fn f2(i: &u64) -> bool {
    i.is_multiple_of(7)
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

    let mut group = c.benchmark_group("xap_f_f");

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
