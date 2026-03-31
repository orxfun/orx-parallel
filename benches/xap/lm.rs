/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_lm/iter/1024        time:   [1.2889 µs 1.2975 µs 1.3064 µs]
xap_lm/xap/1024         time:   [1.4062 µs 1.4182 µs 1.4310 µs]

xap_lm/iter/32768       time:   [45.061 µs 45.492 µs 45.959 µs]
xap_lm/xap/32768        time:   [48.146 µs 48.560 µs 48.984 µs]


SUM BY LOOP:
xap_lm/iter/1024        time:   [3.5383 µs 3.5561 µs 3.5764 µs]
xap_lm/xap/1024         time:   [3.6213 µs 3.6394 µs 3.6590 µs]

xap_lm/iter/32768       time:   [118.21 µs 118.77 µs 119.35 µs]
xap_lm/xap/32768        time:   [119.77 µs 120.53 µs 121.35 µs]


REDUCE:
xap_lm/iter/1024        time:   [2.9425 µs 2.9821 µs 3.0246 µs]
xap_lm/xap/1024         time:   [1.6613 µs 1.6766 µs 1.6939 µs]

xap_lm/iter/32768       time:   [96.032 µs 97.197 µs 98.649 µs]
xap_lm/xap/32768        time:   [54.726 µs 55.290 µs 55.940 µs]


COLLECT:
xap_lm/iter/1024        time:   [7.2635 µs 7.3207 µs 7.3786 µs]
xap_lm/xap/1024         time:   [7.4039 µs 7.4756 µs 7.5464 µs]

xap_lm/iter/32768       time:   [250.35 µs 253.18 µs 256.24 µs]
xap_lm/xap/32768        time:   [255.35 µs 258.15 µs 261.15 µs]


COLLECT BY LOOP:
xap_lm/iter/1024        time:   [5.6661 µs 5.7979 µs 5.9482 µs]
xap_lm/xap/1024         time:   [5.7358 µs 5.8442 µs 5.9757 µs]

xap_lm/iter/32768       time:   [195.45 µs 200.95 µs 206.39 µs]
xap_lm/xap/32768        time:   [202.76 µs 207.56 µs 213.21 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, xap_variants::Id};
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

fn f1(i: u64) -> impl Iterator<Item = u64> {
    (1..7).map(move |x| 3 * x + i + 7)
}

fn f2(i: u64) -> u64 {
    (7 * i).saturating_sub(71)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1).map(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).map(f2);
    let inputs = inputs.iter().copied();
    let iter = inputs.flat_map(|x| xap.xap(x));
    E::out(iter)
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_lm");

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
