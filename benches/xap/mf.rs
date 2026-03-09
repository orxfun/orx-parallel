/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_mf/iter/1024        time:   [666.25 ns 671.84 ns 677.56 ns]
xap_mf/xap/1024         time:   [685.41 ns 689.51 ns 693.80 ns]

xap_mf/iter/32768       time:   [63.374 µs 64.026 µs 64.694 µs]
xap_mf/xap/32768        time:   [73.344 µs 74.537 µs 75.792 µs]

xap_mf/iter/1048576     time:   [3.8060 ms 3.8332 ms 3.8614 ms]
xap_mf/xap/1048576      time:   [3.9554 ms 3.9992 ms 4.0457 ms]


COLLECT:
xap_mf/iter/1024        time:   [1.1552 µs 1.1683 µs 1.1811 µs]
xap_mf/xap/1024         time:   [1.6843 µs 1.6971 µs 1.7094 µs]

xap_mf/iter/32768       time:   [125.43 µs 126.66 µs 128.02 µs]
xap_mf/xap/32768        time:   [119.57 µs 120.64 µs 121.90 µs]

xap_mf/iter/1048576     time:   [5.3821 ms 5.4300 ms 5.4783 ms]
xap_mf/xap/1048576      time:   [4.5187 ms 4.5508 ms 4.5839 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap, XapCopied};
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

fn f1(i: u64) -> u64 {
    2 * i + 1
}

fn f2(i: &u64) -> bool {
    !i.is_multiple_of(3)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().map(f1).filter(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().copied().map(f1).filter(f2);
    E::out(inputs.iter().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_mf");

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
