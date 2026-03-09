/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_mfmfmf/iter/1024    time:   [869.40 ns 877.17 ns 885.91 ns]
xap_mfmfmf/xap/1024     time:   [921.98 ns 926.71 ns 931.85 ns]

xap_mfmfmf/iter/32768   time:   [108.99 µs 109.69 µs 110.40 µs]
xap_mfmfmf/xap/32768    time:   [127.12 µs 127.90 µs 128.68 µs]

xap_mfmfmf/iter/1048576 time:   [5.4004 ms 5.4818 ms 5.5671 ms]
xap_mfmfmf/xap/1048576  time:   [5.0625 ms 5.0981 ms 5.1341 ms]


COLLECT:
xap_mfmfmf/iter/1024    time:   [1.9747 µs 2.0012 µs 2.0319 µs]
xap_mfmfmf/xap/1024     time:   [1.8461 µs 1.8569 µs 1.8683 µs]

xap_mfmfmf/iter/32768   time:   [187.69 µs 191.40 µs 194.45 µs]
xap_mfmfmf/xap/32768    time:   [149.29 µs 150.27 µs 151.34 µs]

xap_mfmfmf/iter/1048576 time:   [6.0972 ms 6.1589 ms 6.2241 ms]
xap_mfmfmf/xap/1048576  time:   [6.2211 ms 6.2646 ms 6.3073 ms]

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

fn f5(i: u64) -> u64 {
    match i.is_multiple_of(2) {
        true => i * 8,
        false => i.saturating_add(7),
    }
}

fn f6(i: &u64) -> bool {
    !i.is_multiple_of(11)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs
        .iter()
        .copied()
        .map(f1)
        .filter(f2)
        .map(f3)
        .filter(f4)
        .map(f5)
        .filter(f6);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new()
        .copied()
        .map(f1)
        .filter(f2)
        .map(f3)
        .filter(f4)
        .map(f5)
        .filter(f6);
    E::out(inputs.iter().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_mfmfmf");

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
