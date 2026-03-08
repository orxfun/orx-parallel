/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_f/iter/1024         time:   [565.08 ns 569.49 ns 573.90 ns]
xap_f/xap/1024          time:   [580.14 ns 583.74 ns 587.38 ns]

xap_f/iter/32768        time:   [84.041 µs 84.737 µs 85.452 µs]
xap_f/xap/32768         time:   [84.901 µs 85.506 µs 86.157 µs]

xap_f/iter/1048576      time:   [3.4204 ms 3.4442 ms 3.4684 ms]
xap_f/xap/1048576       time:   [3.5333 ms 3.5732 ms 3.6159 ms]


COLLECT:
xap_f/iter/1024         time:   [1.0260 µs 1.0359 µs 1.0453 µs]
xap_f/xap/1024          time:   [1.5060 µs 1.5298 µs 1.5518 µs]

xap_f/iter/32768        time:   [120.63 µs 121.66 µs 122.72 µs]
xap_f/xap/32768         time:   [108.21 µs 108.80 µs 109.40 µs]

xap_f/iter/1048576      time:   [4.8007 ms 4.8560 ms 4.9142 ms]
xap_f/xap/1048576       time:   [3.9511 ms 3.9734 ms 3.9955 ms]

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
