/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ffff/iter/1024      time:   [999.31 ns 1.0064 µs 1.0136 µs]
xap_ffff/xap/1024       time:   [1.1636 µs 1.1730 µs 1.1835 µs]

xap_ffff/iter/32768     time:   [143.20 µs 144.31 µs 145.50 µs]
xap_ffff/xap/32768      time:   [139.12 µs 140.86 µs 142.57 µs]

xap_ffff/iter/1048576   time:   [5.7947 ms 5.8373 ms 5.8809 ms]
xap_ffff/xap/1048576    time:   [5.8721 ms 5.9281 ms 5.9856 ms]


COLLECT:
xap_ffff/iter/1024      time:   [1.8440 µs 1.8572 µs 1.8704 µs]
xap_ffff/xap/1024       time:   [2.1215 µs 2.1363 µs 2.1519 µs]

xap_ffff/iter/32768     time:   [214.30 µs 216.03 µs 217.83 µs]
xap_ffff/xap/32768      time:   [213.85 µs 215.66 µs 217.66 µs]

xap_ffff/iter/1048576   time:   [7.8103 ms 7.8636 ms 7.9187 ms]
xap_ffff/xap/1048576    time:   [7.8373 ms 7.8851 ms 7.9345 ms]

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

fn f3(i: &u64) -> bool {
    !(i + 11).is_multiple_of(5)
}

fn f4(i: &u64) -> bool {
    !i.is_multiple_of(5)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs
        .iter()
        .copied()
        .filter(f1)
        .filter(f2)
        .filter(f3)
        .filter(f4);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter(f1).filter(f2).filter(f3).filter(f4);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_ffff");

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
