/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_mmmm/iter/1024      time:   [1.1099 µs 1.1204 µs 1.1322 µs]
xap_mmmm/xap/1024       time:   [1.0832 µs 1.0944 µs 1.1064 µs]

xap_mmmm/iter/32768     time:   [35.724 µs 36.168 µs 36.595 µs]
xap_mmmm/xap/32768      time:   [35.840 µs 36.155 µs 36.504 µs]

xap_mmmm/iter/1048576   time:   [1.1867 ms 1.2019 ms 1.2195 ms]
xap_mmmm/xap/1048576    time:   [1.1694 ms 1.1782 ms 1.1871 ms]


COLLECT:
xap_mmmm/iter/1024      time:   [1.2231 µs 1.2319 µs 1.2408 µs]
xap_mmmm/xap/1024       time:   [1.1817 µs 1.1942 µs 1.2073 µs]

xap_mmmm/iter/32768     time:   [35.568 µs 35.945 µs 36.347 µs]
xap_mmmm/xap/32768      time:   [35.366 µs 35.729 µs 36.104 µs]

xap_mmmm/iter/1048576   time:   [1.1968 ms 1.2078 ms 1.2198 ms]
xap_mmmm/xap/1048576    time:   [1.2159 ms 1.2263 ms 1.2375 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, xap_variants::Id};
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

fn f2(i: u64) -> u64 {
    (7 * i).saturating_sub(71)
}

fn f3(i: u64) -> i64 {
    i as i64 - 33
}

fn f4(i: i64) -> u64 {
    let x = match i < 0 {
        true => (-i) as u64,
        false => i as u64,
    };
    x * 3 + 5
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().map(f1).map(f2).map(f3).map(f4);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().map(f1).map(f2).map(f3).map(f4);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_mmmm");

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
