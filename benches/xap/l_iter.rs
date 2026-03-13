/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_l_iter/iter/1024    time:   [1.1321 µs 1.1379 µs 1.1438 µs]
xap_l_iter/xap/1024     time:   [1.1099 µs 1.1149 µs 1.1200 µs]

xap_l_iter/iter/32768   time:   [37.220 µs 37.489 µs 37.786 µs]
xap_l_iter/xap/32768    time:   [39.664 µs 40.230 µs 40.849 µs]


SUM BY LOOP:
xap_l_iter/iter/1024    time:   [4.1800 µs 4.2219 µs 4.2652 µs]
xap_l_iter/xap/1024     time:   [4.4514 µs 4.5136 µs 4.5841 µs]

xap_l_iter/iter/32768   time:   [139.69 µs 140.84 µs 142.04 µs]
xap_l_iter/xap/32768    time:   [146.05 µs 148.90 µs 151.97 µs]


REDUCE:
xap_l_iter/iter/1024    time:   [2.6005 µs 2.6286 µs 2.6612 µs]
xap_l_iter/xap/1024     time:   [2.5418 µs 2.5556 µs 2.5714 µs]

xap_l_iter/iter/32768   time:   [87.100 µs 88.114 µs 89.183 µs]
xap_l_iter/xap/32768    time:   [89.318 µs 90.511 µs 91.801 µs]


COLLECT:
xap_l_iter/iter/1024    time:   [7.7716 µs 7.8381 µs 7.9201 µs]
xap_l_iter/xap/1024     time:   [8.2068 µs 8.2536 µs 8.3002 µs]

xap_l_iter/iter/32768   time:   [249.41 µs 250.99 µs 252.61 µs]
xap_l_iter/xap/32768    time:   [258.36 µs 260.98 µs 263.97 µs]


COLLECT BY LOOP:
xap_l_iter/iter/1024    time:   [7.9477 µs 8.0279 µs 8.1125 µs]
xap_l_iter/xap/1024     time:   [8.5372 µs 8.6432 µs 8.7612 µs]

xap_l_iter/iter/32768   time:   [270.55 µs 275.73 µs 280.87 µs]
xap_l_iter/xap/32768    time:   [262.64 µs 266.44 µs 270.39 µs]

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
        .filter(|x| !x.is_multiple_of(999))
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1);
    let inputs = inputs.iter().copied();
    let iter = XapIter::new(inputs, xap);
    E::out(iter)
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
