/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_lll_iter/iter/1024  time:   [8.8103 µs 8.9234 µs 9.0562 µs]
xap_lll_iter/xap/1024   time:   [53.149 µs 53.566 µs 54.020 µs]

xap_lll_iter/iter/32768 time:   [271.47 µs 274.28 µs 276.94 µs]
xap_lll_iter/xap/32768  time:   [1.8339 ms 1.8521 ms 1.8711 ms]

SUM BY LOOP:
xap_lll_iter/iter/1024  time:   [52.304 µs 52.837 µs 53.388 µs]
xap_lll_iter/xap/1024   time:   [51.773 µs 52.451 µs 53.123 µs]

xap_lll_iter/iter/32768 time:   [1.6673 ms 1.6863 ms 1.7072 ms]
xap_lll_iter/xap/32768  time:   [1.6089 ms 1.6216 ms 1.6353 ms]


COLLECT:
xap_lll_iter/iter/1024  time:   [74.686 µs 75.528 µs 76.491 µs]
xap_lll_iter/xap/1024   time:   [92.915 µs 94.745 µs 96.674 µs]

xap_lll_iter/iter/32768 time:   [2.3660 ms 2.3893 ms 2.4143 ms]
xap_lll_iter/xap/32768  time:   [2.8669 ms 2.8932 ms 2.9210 ms]

COLLECT BY LOOP:
xap_lll_iter/iter/1024  time:   [71.586 µs 73.142 µs 74.735 µs]
xap_lll_iter/xap/1024   time:   [71.181 µs 73.001 µs 75.035 µs]

xap_lll_iter/iter/32768 time:   [2.2528 ms 2.2728 ms 2.2958 ms]
xap_lll_iter/xap/32768  time:   [2.5069 ms 2.5482 ms 2.5897 ms]

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

fn f2(i: u64) -> impl IntoIterator<Item = u64> {
    (6..8)
        .map(move |x| i + 5 * x as u64 + 2)
        .filter(|x| !x.is_multiple_of(999))
}

fn f3(i: u64) -> impl IntoIterator<Item = u64> {
    (9..12)
        .map(move |x| i + 3 * x as u64 + 7)
        .filter(|x| !x.is_multiple_of(999))
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
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
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
