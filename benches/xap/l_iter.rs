/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_l_iter/iter/1024    time:   [1.6813 µs 1.7038 µs 1.7283 µs]
xap_l_iter/xap/1024     time:   [1.6633 µs 1.6817 µs 1.7013 µs]

xap_l_iter/iter/32768   time:   [51.707 µs 52.298 µs 52.879 µs]
xap_l_iter/xap/32768    time:   [51.009 µs 51.629 µs 52.305 µs]

SUM BY LOOP:
xap_l_iter/iter/1024    time:   [8.3933 µs 8.6995 µs 9.0369 µs]
xap_l_iter/xap/1024     time:   [13.020 µs 16.564 µs 21.273 µs]

xap_l_iter/iter/32768   time:   [194.15 µs 197.77 µs 201.54 µs]
xap_l_iter/xap/32768    time:   [157.24 µs 159.64 µs 162.60 µs]


COLLECT:
xap_l_iter/iter/1024    time:   [9.3365 µs 9.4599 µs 9.5919 µs]
xap_l_iter/xap/1024     time:   [10.686 µs 10.898 µs 11.099 µs]

xap_l_iter/iter/32768   time:   [306.29 µs 310.32 µs 314.71 µs]
xap_l_iter/xap/32768    time:   [301.43 µs 305.28 µs 309.17 µs]

COLLECT BY LOOP:
xap_l_iter/iter/1024    time:   [8.8989 µs 9.0173 µs 9.1317 µs]
xap_l_iter/xap/1024     time:   [9.3978 µs 9.6033 µs 9.8269 µs]

xap_l_iter/iter/32768   time:   [316.53 µs 323.41 µs 330.95 µs]
xap_l_iter/xap/32768    time:   [322.93 µs 331.48 µs 339.96 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = CollectByLoop;

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

pub struct SumByLoop;
impl Exp for SumByLoop {
    type Out = u64;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        let mut v = 0;
        for x in i {
            v += x;
        }
        v
    }
}

pub struct Collect;
impl Exp for Collect {
    type Out = Vec<u64>;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        i.collect()
    }
}

pub struct CollectByLoop;
impl Exp for CollectByLoop {
    type Out = Vec<u64>;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        let mut v = Vec::new();
        for x in i {
            v.push(x);
        }
        v
    }
}

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f1(i: u64) -> impl IntoIterator<Item = u64> {
    (2..5)
        .map(move |x| i + 2 * x as u64 + 5)
        .filter(|x| !x.is_multiple_of(999))
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_l_iter");

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
