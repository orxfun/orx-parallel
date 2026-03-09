/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_lll/iter/1024       time:   [22.600 µs 22.869 µs 23.155 µs]
xap_lll/xap/1024        time:   [182.10 µs 183.24 µs 184.47 µs]

xap_lll/iter/32768      time:   [723.28 µs 728.27 µs 733.65 µs]
xap_lll/xap/32768       time:   [6.0452 ms 6.0994 ms 6.1562 ms]

xap_lll/iter/1048576    time:   [25.259 ms 25.629 ms 26.007 ms]
xap_lll/xap/1048576     time:   [216.81 ms 219.13 ms 221.57 ms]


COLLECT:
xap_lll/iter/1024       time:   [27.571 µs 27.926 µs 28.305 µs]
xap_lll/xap/1024        time:   [230.13 µs 232.29 µs 234.61 µs]

xap_lll/iter/32768      time:   [1.1278 ms 1.1399 ms 1.1524 ms]
xap_lll/xap/32768       time:   [17.664 ms 17.789 ms 17.921 ms]

xap_lll/iter/1048576    time:   [338.00 ms 348.38 ms 360.85 ms]
xap_lll/xap/1048576     time:   [618.99 ms 636.57 ms 655.71 ms]

(!) significant difference

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

fn f1(i: u64) -> Vec<u64> {
    vec![i + 1, i * 2, i + 5, i + 4, i, i.saturating_sub(3), 7 * i]
}

fn f2(i: u64) -> Vec<u64> {
    vec![i * 2 + 1, i, i.saturating_sub(7)]
}

fn f3(i: u64) -> Vec<u64> {
    vec![i / 3, i + 7, i.saturating_sub(4), i / 4, i]
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs
        .iter()
        .copied()
        .flat_map(f1)
        .flat_map(f2)
        .flat_map(f3);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).flat_map(f2).flat_map(f3);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_lll");

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
