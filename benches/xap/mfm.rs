/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_mfm/iter/1024       time:   [1.3496 µs 1.3748 µs 1.4044 µs]
xap_mfm/xap/1024        time:   [653.39 ns 659.41 ns 666.01 ns]

xap_mfm/iter/32768      time:   [41.659 µs 42.220 µs 42.904 µs]
xap_mfm/xap/32768       time:   [25.534 µs 25.848 µs 26.177 µs]

xap_mfm/iter/1048576    time:   [1.4188 ms 1.4508 ms 1.4916 ms]
xap_mfm/xap/1048576     time:   [727.55 µs 737.02 µs 746.66 µs]


COLLECT:
xap_mfm/iter/1024       time:   [1.1205 µs 1.1272 µs 1.1341 µs]
xap_mfm/xap/1024        time:   [1.7840 µs 1.7959 µs 1.8083 µs]

xap_mfm/iter/32768      time:   [147.15 µs 148.61 µs 150.10 µs]
xap_mfm/xap/32768       time:   [130.71 µs 131.65 µs 132.58 µs]

xap_mfm/iter/1048576    time:   [5.3197 ms 5.3562 ms 5.3939 ms]
xap_mfm/xap/1048576     time:   [4.9456 ms 4.9866 ms 5.0284 ms]

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

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().map(f1).filter(f2).map(f3);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().copied().map(f1).filter(f2).map(f3);
    E::out(inputs.iter().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_mfm");

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
