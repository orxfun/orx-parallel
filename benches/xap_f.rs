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
xap_f/iter/1024         time:   [806.98 ns 814.09 ns 821.95 ns]
xap_f/xap/1024          time:   [1.2381 µs 1.2449 µs 1.2516 µs]

xap_f/iter/32768        time:   [72.516 µs 72.949 µs 73.394 µs]
xap_f/xap/32768         time:   [74.432 µs 74.778 µs 75.170 µs]

xap_f/iter/1048576      time:   [2.8601 ms 2.8767 ms 2.8943 ms]
xap_f/xap/1048576       time:   [2.9920 ms 3.0104 ms 3.0292 ms]

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

fn f(i: &u64) -> bool {
    !(2 * i + 1).is_multiple_of(5)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter(f);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter(f);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_f");

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
