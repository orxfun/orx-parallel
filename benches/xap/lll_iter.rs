/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_lll_iter/iter/1024  time:   [3.5231 µs 3.5465 µs 3.5713 µs]
xap_lll_iter/xap/1024   time:   [3.4614 µs 3.4829 µs 3.5053 µs]

xap_lll_iter/iter/32768 time:   [286.74 µs 291.74 µs 296.92 µs]
xap_lll_iter/xap/32768  time:   [292.21 µs 294.84 µs 297.84 µs]


SUM BY LOOP:
xap_lll_iter/iter/1024  time:   [19.139 µs 19.294 µs 19.458 µs]
xap_lll_iter/xap/1024   time:   [18.689 µs 18.846 µs 19.007 µs]

xap_lll_iter/iter/32768 time:   [859.88 µs 871.83 µs 884.87 µs]
xap_lll_iter/xap/32768  time:   [807.08 µs 813.74 µs 821.26 µs]


REDUCE:
xap_lll_iter/iter/1024  time:   [22.493 µs 22.821 µs 23.157 µs]
xap_lll_iter/xap/1024   time:   [24.253 µs 24.452 µs 24.662 µs]

xap_lll_iter/iter/32768 time:   [791.58 µs 796.33 µs 801.31 µs]
xap_lll_iter/xap/32768  time:   [792.81 µs 799.48 µs 806.52 µs]


COLLECT:
xap_lll_iter/iter/1024  time:   [30.074 µs 30.571 µs 31.137 µs]
xap_lll_iter/xap/1024   time:   [29.526 µs 30.033 µs 30.603 µs]

xap_lll_iter/iter/32768 time:   [957.74 µs 964.16 µs 971.09 µs]
xap_lll_iter/xap/32768  time:   [1.0266 ms 1.0331 ms 1.0403 ms]


COLLECT BY LOOP:
xap_lll_iter/iter/1024  time:   [25.912 µs 26.305 µs 26.714 µs]
xap_lll_iter/xap/1024   time:   [26.214 µs 26.558 µs 26.905 µs]

xap_lll_iter/iter/32768 time:   [1.0124 ms 1.0270 ms 1.0411 ms]
xap_lll_iter/xap/32768  time:   [963.01 µs 974.94 µs 988.67 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, xap_variants::Id};
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
    let iter = inputs.flat_map(|x| xap.xap(x));
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
