/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_i/iter/1024         time:   [492.96 ns 502.37 ns 510.84 ns]
xap_i/xap/1024          time:   [506.02 ns 511.53 ns 517.32 ns]

xap_i/iter/32768        time:   [17.248 µs 17.578 µs 17.938 µs]
xap_i/xap/32768         time:   [16.838 µs 17.134 µs 17.463 µs]

xap_i/iter/1048576      time:   [1.7246 ms 1.7401 ms 1.7579 ms]
xap_i/xap/1048576       time:   [1.7432 ms 1.7598 ms 1.7785 ms]


COLLECT:
xap_i/iter/1024         time:   [1.0152 µs 1.0232 µs 1.0315 µs]
xap_i/xap/1024          time:   [1.1702 µs 1.1786 µs 1.1875 µs]

xap_i/iter/32768        time:   [64.285 µs 64.678 µs 65.104 µs]
xap_i/xap/32768         time:   [69.470 µs 69.988 µs 70.547 µs]

xap_i/iter/1048576      time:   [2.4129 ms 2.4319 ms 2.4527 ms]
xap_i/xap/1048576       time:   [2.5399 ms 2.5614 ms 2.5839 ms]

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

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter_map(f1);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter_map(f1);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_i");

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
