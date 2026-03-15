/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_p_iii_vec/iter/1024 time:   [826.34 ns 834.16 ns 842.89 ns]
xap_p_iii_vec/xap/1024  time:   [913.46 ns 921.93 ns 931.05 ns]

xap_p_iii_vec/iter/32768time:   [116.70 µs 117.97 µs 119.36 µs]
xap_p_iii_vec/xap/32768 time:   [90.196 µs 91.484 µs 92.901 µs]


SUM BY LOOP:
xap_p_iii_vec/iter/1024 time:   [818.29 ns 825.08 ns 832.62 ns]
xap_p_iii_vec/xap/1024  time:   [634.45 ns 638.10 ns 642.18 ns]

xap_p_iii_vec/iter/32768time:   [90.792 µs 92.157 µs 93.507 µs]
xap_p_iii_vec/xap/32768 time:   [117.22 µs 118.06 µs 118.91 µs]


REDUCE:
xap_p_iii_vec/iter/1024 time:   [1.0085 µs 1.0176 µs 1.0269 µs]
xap_p_iii_vec/xap/1024  time:   [889.05 ns 894.62 ns 900.84 ns]

xap_p_iii_vec/iter/32768time:   [105.35 µs 106.13 µs 106.91 µs]
xap_p_iii_vec/xap/32768 time:   [111.24 µs 112.62 µs 114.08 µs]


COLLECT:
xap_p_iii_vec/iter/1024 time:   [1.2201 µs 1.2288 µs 1.2380 µs]
xap_p_iii_vec/xap/1024  time:   [1.1967 µs 1.2032 µs 1.2106 µs]

xap_p_iii_vec/iter/32768time:   [117.04 µs 118.78 µs 120.54 µs]
xap_p_iii_vec/xap/32768 time:   [96.255 µs 97.259 µs 98.315 µs]


COLLECT BY LOOP:
xap_p_iii_vec/iter/1024 time:   [1.3477 µs 1.3536 µs 1.3598 µs]
xap_p_iii_vec/xap/1024  time:   [1.3993 µs 1.4077 µs 1.4164 µs]

xap_p_iii_vec/iter/32768time:   [118.10 µs 119.39 µs 120.70 µs]
xap_p_iii_vec/xap/32768 time:   [108.28 µs 109.69 µs 111.21 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = Sum;

trait Exp {
    type Out;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>;
}

pub struct Sum;
impl Exp for Sum {
    type Out = u64;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>,
    {
        let mut x = 0;
        for i in inputs {
            let i = black_box(i);
            x += fmap(i).into_iter().sum::<u64>();
        }
        x
    }
}

pub struct SumByLoop;
impl Exp for SumByLoop {
    type Out = u64;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>,
    {
        let mut x = 0;
        for i in inputs {
            let i = black_box(i);
            for j in fmap(i).into_iter() {
                x += j;
            }
        }
        x
    }
}

pub struct Reduce;
impl Exp for Reduce {
    type Out = Option<u64>;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>,
    {
        let mut x = None;
        for i in inputs {
            let i = black_box(i);
            if let Some(y) = fmap(i).into_iter().reduce(|x, y| 2 * x + y + 7) {
                x = match &mut x {
                    None => Some(y),
                    Some(x) => Some(2 * *x + y + 7),
                };
            }
        }
        x
    }
}

pub struct Collect;
impl Exp for Collect {
    type Out = Vec<u64>;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>,
    {
        let mut x = vec![];
        for i in inputs {
            let i = black_box(i);
            x.extend(fmap(i));
        }
        x
    }
}

pub struct CollectByLoop;
impl Exp for CollectByLoop {
    type Out = Vec<u64>;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>,
    {
        let mut x = vec![];
        for i in inputs {
            let i = black_box(i);
            for j in fmap(i) {
                x.push(j);
            }
        }
        x
    }
}

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f1(i: u64) -> Option<u64> {
    match i.is_multiple_of(7) {
        true => None,
        false => Some(i + 3),
    }
}

fn f2(i: u64) -> Option<u64> {
    match i.is_multiple_of(3) {
        true => None,
        false => Some(2 * i),
    }
}

fn f3(i: u64) -> Option<u64> {
    match (i + 5).is_multiple_of(4) {
        true => None,
        false => Some(3 * i + 1),
    }
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied();
    E::out(iter, |i| f1(i).into_iter().filter_map(f2).filter_map(f3))
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter_map(f1).filter_map(f2).filter_map(f3);
    let iter = inputs.iter().copied();
    E::out(iter, |i| xap.xap(i))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_p_iii_vec");

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
