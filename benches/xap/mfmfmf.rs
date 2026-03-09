/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_mfmfmf/iter/1024    time:   [882.31 ns 889.14 ns 896.86 ns]
xap_mfmfmf/xap/1024     time:   [1.5259 µs 1.5335 µs 1.5409 µs]

xap_mfmfmf/iter/32768   time:   [119.47 µs 120.16 µs 120.92 µs]
xap_mfmfmf/xap/32768    time:   [124.69 µs 125.44 µs 126.27 µs]

xap_mfmfmf/iter/1048576 time:   [4.7783 ms 4.8042 ms 4.8305 ms]
xap_mfmfmf/xap/1048576  time:   [4.7873 ms 4.8204 ms 4.8549 ms]


COLLECT:
xap_mfmfmf/iter/1024    time:   [1.8504 µs 1.8751 µs 1.8993 µs]
xap_mfmfmf/xap/1024     time:   [2.1298 µs 2.1470 µs 2.1638 µs]

xap_mfmfmf/iter/32768   time:   [154.21 µs 155.23 µs 156.25 µs]
xap_mfmfmf/xap/32768    time:   [149.16 µs 150.39 µs 151.71 µs]

xap_mfmfmf/iter/1048576 time:   [5.3506 ms 5.3758 ms 5.4013 ms]
xap_mfmfmf/xap/1048576  time:   [6.1129 ms 6.1984 ms 6.2895 ms]

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
