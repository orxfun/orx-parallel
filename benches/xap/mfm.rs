/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_mfm/iter/1024       time:   [1.1767 µs 1.1828 µs 1.1890 µs]
xap_mfm/xap/1024        time:   [659.36 ns 664.33 ns 669.38 ns]

xap_mfm/iter/32768      time:   [39.520 µs 39.715 µs 39.924 µs]
xap_mfm/xap/32768       time:   [22.058 µs 22.290 µs 22.538 µs]

xap_mfm/iter/1048576    time:   [1.2449 ms 1.2548 ms 1.2664 ms]
xap_mfm/xap/1048576     time:   [701.86 µs 710.64 µs 719.88 µs]


COLLECT:
xap_mfm/iter/1024       time:   [1.0360 µs 1.0438 µs 1.0519 µs]
xap_mfm/xap/1024        time:   [1.6564 µs 1.6668 µs 1.6775 µs]

xap_mfm/iter/32768      time:   [134.73 µs 136.17 µs 137.75 µs]
xap_mfm/xap/32768       time:   [116.54 µs 117.88 µs 119.23 µs]

xap_mfm/iter/1048576    time:   [4.8884 ms 4.9217 ms 4.9560 ms]
xap_mfm/xap/1048576     time:   [4.3721 ms 4.4028 ms 4.4344 ms]

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
