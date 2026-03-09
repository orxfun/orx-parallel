/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_l/iter/1024         time:   [570.24 ns 574.28 ns 578.70 ns]
xap_l/xap/1024          time:   [577.08 ns 580.14 ns 583.27 ns]

xap_l/iter/32768        time:   [18.828 µs 18.919 µs 19.012 µs]
xap_l/xap/32768         time:   [18.891 µs 19.036 µs 19.201 µs]

xap_l/iter/1048576      time:   [616.97 µs 620.40 µs 623.79 µs]
xap_l/xap/1048576       time:   [618.28 µs 622.01 µs 625.71 µs]


COLLECT:
xap_l/iter/1024         time:   [1.6761 µs 1.7021 µs 1.7317 µs]
xap_l/xap/1024          time:   [1.5494 µs 1.5671 µs 1.5866 µs]

xap_l/iter/32768        time:   [51.823 µs 52.238 µs 52.681 µs]
xap_l/xap/32768         time:   [53.788 µs 54.402 µs 54.991 µs]

xap_l/iter/1048576      time:   [20.811 ms 21.039 ms 21.289 ms]
xap_l/xap/1048576       time:   [21.004 ms 21.165 ms 21.344 ms]

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

fn f1(i: u64) -> [u64; 7] {
    [i + 1, i * 2, i + 5, i + 4, i, i.saturating_sub(3), 7 * i]
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_l");

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
