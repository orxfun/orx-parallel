/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_mfmf/iter/1024      time:   [949.13 ns 954.80 ns 961.02 ns]
xap_mfmf/xap/1024       time:   [976.99 ns 987.81 ns 997.66 ns]

xap_mfmf/iter/32768     time:   [112.00 µs 113.17 µs 114.35 µs]
xap_mfmf/xap/32768      time:   [124.85 µs 125.55 µs 126.27 µs]

xap_mfmf/iter/1048576   time:   [4.4079 ms 4.4533 ms 4.5039 ms]
xap_mfmf/xap/1048576    time:   [5.1821 ms 5.2573 ms 5.3348 ms]


COLLECT:
xap_mfmf/iter/1024      time:   [1.3819 µs 1.3950 µs 1.4084 µs]
xap_mfmf/xap/1024       time:   [1.8585 µs 1.8710 µs 1.8833 µs]

xap_mfmf/iter/32768     time:   [153.73 µs 155.31 µs 157.05 µs]
xap_mfmf/xap/32768      time:   [132.96 µs 133.58 µs 134.26 µs]

xap_mfmf/iter/1048576   time:   [5.4275 ms 5.4637 ms 5.5036 ms]
xap_mfmf/xap/1048576    time:   [5.2033 ms 5.2316 ms 5.2596 ms]

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

fn f3(i: u64) -> u64 {
    i * 7 + 2
}

fn f4(i: &u64) -> bool {
    !i.is_multiple_of(7)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().map(f1).filter(f2).map(f3).filter(f4);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().copied().map(f1).filter(f2).map(f3).filter(f4);
    E::out(inputs.iter().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_mfmf");

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
