/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_l_iter/iter/1024    time:   [1.4628 µs 1.4806 µs 1.4999 µs]
xap_l_iter/xap/1024     time:   [1.4639 µs 1.4791 µs 1.4937 µs]

xap_l_iter/iter/32768   time:   [45.877 µs 46.229 µs 46.608 µs]
xap_l_iter/xap/32768    time:   [48.273 µs 49.007 µs 49.839 µs]

SUM BY LOOP:
xap_l_iter/iter/1024    time:   [4.9615 µs 5.0194 µs 5.0754 µs]
xap_l_iter/xap/1024     time:   [4.4484 µs 4.4880 µs 4.5345 µs]

xap_l_iter/iter/32768   time:   [139.09 µs 140.11 µs 141.27 µs]
xap_l_iter/xap/32768    time:   [137.99 µs 139.06 µs 140.25 µs]


REDUCE:
xap_l_iter/iter/1024    time:   [2.9544 µs 3.0056 µs 3.0604 µs]
xap_l_iter/xap/1024     time:   [2.9493 µs 3.0105 µs 3.0819 µs]

xap_l_iter/iter/32768   time:   [91.712 µs 93.662 µs 95.900 µs]
xap_l_iter/xap/32768    time:   [92.291 µs 93.381 µs 94.559 µs]


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

type Output = Reduce;

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

pub struct Reduce;
impl Exp for Reduce {
    type Out = Option<u64>;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        i.reduce(|x, y| 2 * x + y + 7)
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
