/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_fff/iter/1024       time:   [986.59 ns 993.25 ns 1.0003 µs]
xap_fff/xap/1024        time:   [1.1239 µs 1.1449 µs 1.1642 µs]

xap_fff/iter/32768      time:   [135.40 µs 137.48 µs 139.83 µs]
xap_fff/xap/32768       time:   [109.90 µs 111.62 µs 113.68 µs]

xap_fff/iter/1048576    time:   [4.8968 ms 4.9689 ms 5.0483 ms]
xap_fff/xap/1048576     time:   [5.8498 ms 6.0038 ms 6.1642 ms]


COLLECT:
xap_fff/iter/1024       time:   [1.8100 µs 1.8242 µs 1.8386 µs]
xap_fff/xap/1024        time:   [2.0857 µs 2.1124 µs 2.1407 µs]

xap_fff/iter/32768      time:   [192.00 µs 194.67 µs 197.49 µs]
xap_fff/xap/32768       time:   [188.43 µs 190.74 µs 193.02 µs]

xap_fff/iter/1048576    time:   [7.1527 ms 7.2340 ms 7.3226 ms]
xap_fff/xap/1048576     time:   [8.4760 ms 8.6209 ms 8.7720 ms]

TODO: room for performance improvement

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

fn f2(i: &u64) -> bool {
    !(i + 7).is_multiple_of(11)
}

fn f3(i: &u64) -> bool {
    !(i + 11).is_multiple_of(5)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter(f1).filter(f2).filter(f3);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter(f1).filter(f2).filter(f3);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_fff");

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
