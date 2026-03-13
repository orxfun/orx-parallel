/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_lll_iter/iter/1024  time:   [4.1900 µs 4.2234 µs 4.2564 µs]
xap_lll_iter/xap/1024   time:   [3.6110 µs 3.6381 µs 3.6661 µs]

xap_lll_iter/iter/32768 time:   [334.20 µs 339.49 µs 345.29 µs]
xap_lll_iter/xap/32768  time:   [322.71 µs 328.01 µs 333.75 µs]


SUM BY LOOP:
xap_lll_iter/iter/1024  time:   [17.814 µs 18.000 µs 18.213 µs]
xap_lll_iter/xap/1024   time:   [19.878 µs 19.992 µs 20.115 µs]

xap_lll_iter/iter/32768 time:   [792.19 µs 800.05 µs 808.81 µs]
xap_lll_iter/xap/32768  time:   [855.71 µs 863.97 µs 872.39 µs]


REDUCE:
xap_lll_iter/iter/1024  time:   [21.644 µs 21.811 µs 21.994 µs]
xap_lll_iter/xap/1024   time:   [22.881 µs 23.006 µs 23.139 µs]

xap_lll_iter/iter/32768 time:   [779.33 µs 784.09 µs 789.22 µs]
xap_lll_iter/xap/32768  time:   [797.54 µs 803.38 µs 809.74 µs]


COLLECT:
xap_lll_iter/iter/1024  time:   [28.772 µs 29.028 µs 29.262 µs]
xap_lll_iter/xap/1024   time:   [28.712 µs 28.878 µs 29.042 µs]

xap_lll_iter/iter/32768 time:   [1.0142 ms 1.0244 ms 1.0355 ms]
xap_lll_iter/xap/32768  time:   [1.0475 ms 1.0553 ms 1.0645 ms]


COLLECT BY LOOP:
xap_lll_iter/iter/1024  time:   [24.704 µs 24.915 µs 25.131 µs]
xap_lll_iter/xap/1024   time:   [25.601 µs 25.829 µs 26.052 µs]

xap_lll_iter/iter/32768 time:   [894.48 µs 900.05 µs 906.25 µs]
xap_lll_iter/xap/32768  time:   [916.48 µs 921.71 µs 927.42 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap, xap_iter::XapIter};
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
        .filter(|x| !x.is_multiple_of(3))
}

fn f2(i: u64) -> impl IntoIterator<Item = u64> {
    (6..8)
        .map(move |x| i + 5 * x as u64 + 2)
        .filter(|x| !x.is_multiple_of(3))
}

fn f3(i: u64) -> impl IntoIterator<Item = u64> {
    (9..12)
        .map(move |x| i + 3 * x as u64 + 7)
        .filter(|x| !x.is_multiple_of(3))
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs
        .iter()
        .copied()
        .flat_map(f1)
        .flat_map(f2)
        .flat_map(f3);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).flat_map(f2).flat_map(f3);
    let inputs = inputs.iter().copied();
    let iter = XapIter::new(inputs, xap);
    E::out(iter)
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_lll_iter");

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
