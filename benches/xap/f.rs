/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_f/iter/1024         time:   [522.96 ns 526.22 ns 529.81 ns]
xap_f/xap/1024          time:   [529.05 ns 531.50 ns 534.01 ns]

xap_f/iter/32768        time:   [79.767 µs 80.427 µs 81.127 µs]
xap_f/xap/32768         time:   [79.418 µs 79.976 µs 80.593 µs]

xap_f/iter/1048576      time:   [3.2563 ms 3.2871 ms 3.3233 ms]
xap_f/xap/1048576       time:   [3.3664 ms 3.3969 ms 3.4298 ms]


COLLECT:
xap_f/iter/1024         time:   [1.3871 µs 1.4086 µs 1.4337 µs]
xap_f/xap/1024          time:   [1.8211 µs 1.8405 µs 1.8619 µs]

xap_f/iter/32768        time:   [162.65 µs 164.15 µs 165.61 µs]
xap_f/xap/32768         time:   [166.84 µs 168.93 µs 170.91 µs]

xap_f/iter/1048576      time:   [6.5626 ms 6.6888 ms 6.8133 ms]
xap_f/xap/1048576       time:   [5.1423 ms 5.2626 ms 5.3816 ms]

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

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter(f1);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter(f1);
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
