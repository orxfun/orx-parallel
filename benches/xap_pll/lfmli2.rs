/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_p_lfmli2/iter/1024  time:   [10.430 µs 10.570 µs 10.709 µs]
xap_p_lfmli2/xap/1024   time:   [8.7429 µs 8.8293 µs 8.9148 µs]

xap_p_lfmli2/iter/32768 time:   [468.58 µs 472.54 µs 476.60 µs]
xap_p_lfmli2/xap/32768  time:   [446.32 µs 449.27 µs 452.27 µs]


REDUCE:
xap_p_lfmli2/iter/1024  time:   [7.9578 µs 8.0405 µs 8.1255 µs]
xap_p_lfmli2/xap/1024   time:   [10.212 µs 10.286 µs 10.360 µs]

xap_p_lfmli2/iter/32768 time:   [379.99 µs 381.90 µs 383.93 µs]
xap_p_lfmli2/xap/32768  time:   [463.77 µs 466.08 µs 468.50 µs]


COLLECT:
xap_p_lfmli2/iter/1024  time:   [37.105 µs 37.415 µs 37.718 µs]
xap_p_lfmli2/xap/1024   time:   [41.655 µs 42.289 µs 42.858 µs]

xap_p_lfmli2/iter/32768 time:   [1.4662 ms 1.4818 ms 1.4963 ms]
xap_p_lfmli2/xap/32768  time:   [1.4064 ms 1.4189 ms 1.4323 ms]


*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, xap_variants::Id};
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

fn f1(i: u64) -> impl Iterator<Item = u64> {
    (1..7).map(move |x| 3 * x + i + 7)
}

fn f2(i: &u64) -> bool {
    !(i + 7).is_multiple_of(11)
}

fn f3(i: u64) -> u64 {
    i * 3 + 5
}

fn f4(i: u64) -> impl Iterator<Item = u64> {
    (9..12).map(move |x| x + 2 * i + 11)
}

fn f5(i: u64) -> Option<u64> {
    match (i + 5).is_multiple_of(4) {
        true => None,
        false => Some(3 * i + 1),
    }
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied();
    E::out(iter, |i| {
        f1(i)
            .into_iter()
            .filter(f2)
            .map(f3)
            .flat_map(f4)
            .filter_map(f5)
    })
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new()
        .flat_map(f1)
        .filter(f2)
        .map(f3)
        .flat_map(f4)
        .filter_map(f5);
    let iter = inputs.iter().copied();
    E::out(iter, |i| xap.xap(i))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_p_lfmli2");

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
