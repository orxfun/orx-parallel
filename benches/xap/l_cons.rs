/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_l_cons/iter/1024    time:   [749.31 ns 762.18 ns 775.76 ns]
xap_l_cons/xap/1024     time:   [751.94 ns 768.08 ns 784.68 ns]

xap_l_cons/iter/32768   time:   [23.237 µs 23.622 µs 24.059 µs]
xap_l_cons/xap/32768    time:   [22.554 µs 22.917 µs 23.336 µs]


SUM BY LOOP:
xap_l_cons/iter/1024    time:   [8.9517 µs 9.1179 µs 9.2785 µs]
xap_l_cons/xap/1024     time:   [8.3582 µs 8.4604 µs 8.5538 µs]

xap_l_cons/iter/32768   time:   [271.54 µs 275.76 µs 279.82 µs]
xap_l_cons/xap/32768    time:   [262.75 µs 265.45 µs 268.06 µs]


REDUCE:
xap_l_cons/iter/1024    time:   [1.5395 µs 1.5559 µs 1.5766 µs]
xap_l_cons/xap/1024     time:   [1.7007 µs 1.7391 µs 1.7784 µs]

xap_l_cons/iter/32768   time:   [56.352 µs 57.062 µs 57.818 µs]
xap_l_cons/xap/32768    time:   [56.664 µs 57.530 µs 58.448 µs]


COLLECT:
xap_l_cons/iter/1024    time:   [1.8938 µs 1.9165 µs 1.9429 µs]
xap_l_cons/xap/1024     time:   [1.8867 µs 1.9149 µs 1.9452 µs]

xap_l_cons/iter/32768   time:   [62.192 µs 63.262 µs 64.496 µs]
xap_l_cons/xap/32768    time:   [76.693 µs 79.778 µs 82.525 µs]


COLLECT BY LOOP:
xap_l_cons/iter/1024    time:   [28.986 µs 29.199 µs 29.419 µs]
xap_l_cons/xap/1024     time:   [29.816 µs 30.018 µs 30.241 µs]

xap_l_cons/iter/32768   time:   [999.14 µs 1.0180 ms 1.0396 ms]
xap_l_cons/xap/32768    time:   [1.1383 ms 1.1582 ms 1.1783 ms]

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

fn f1(i: u64) -> [u64; 7] {
    [i + 1, i * 2, i + 5, i + 4, i, i.saturating_sub(3), 7 * i]
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1);
    let inputs = inputs.iter().copied();
    let iter = inputs.flat_map(|x| xap.xap(x));
    E::out(iter)
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_l_cons");

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
