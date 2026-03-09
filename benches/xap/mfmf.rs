/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_mfmf/iter/1024      time:   [801.41 ns 809.07 ns 818.43 ns]
xap_mfmf/xap/1024       time:   [853.43 ns 859.70 ns 866.68 ns]

xap_mfmf/iter/32768     time:   [101.10 µs 101.73 µs 102.41 µs]
xap_mfmf/xap/32768      time:   [110.80 µs 112.05 µs 113.43 µs]

xap_mfmf/iter/1048576   time:   [3.9718 ms 4.0201 ms 4.0712 ms]
xap_mfmf/xap/1048576    time:   [4.2536 ms 4.2835 ms 4.3159 ms]


COLLECT:
xap_mfmf/iter/1024      time:   [1.4129 µs 1.4236 µs 1.4351 µs]
xap_mfmf/xap/1024       time:   [1.8499 µs 1.8612 µs 1.8728 µs]

xap_mfmf/iter/32768     time:   [126.32 µs 127.28 µs 128.23 µs]
xap_mfmf/xap/32768      time:   [131.20 µs 132.04 µs 132.91 µs]

xap_mfmf/iter/1048576   time:   [4.5168 ms 4.5428 ms 4.5697 ms]
xap_mfmf/xap/1048576    time:   [4.9037 ms 4.9312 ms 4.9589 ms]

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
