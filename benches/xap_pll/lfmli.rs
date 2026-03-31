/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_p_lfmli/iter/1024   time:   [10.919 µs 11.121 µs 11.309 µs]
xap_p_lfmli/xap/1024    time:   [23.799 µs 23.918 µs 24.037 µs]

xap_p_lfmli/iter/32768  time:   [483.98 µs 487.34 µs 490.91 µs]
xap_p_lfmli/xap/32768   time:   [1.0730 ms 1.0789 ms 1.0854 ms]


SUM BY LOOP:
xap_p_lfmli/iter/1024   time:   [16.689 µs 16.790 µs 16.896 µs]
xap_p_lfmli/xap/1024    time:   [20.651 µs 20.784 µs 20.925 µs]

xap_p_lfmli/iter/32768  time:   [878.30 µs 883.20 µs 887.90 µs]
xap_p_lfmli/xap/32768   time:   [967.01 µs 972.85 µs 979.23 µs]


REDUCE:
xap_p_lfmli/iter/1024   time:   [10.296 µs 10.378 µs 10.456 µs]
xap_p_lfmli/xap/1024    time:   [33.074 µs 33.370 µs 33.700 µs]

xap_p_lfmli/iter/32768  time:   [490.38 µs 492.98 µs 495.58 µs]
xap_p_lfmli/xap/32768   time:   [1.4418 ms 1.4486 ms 1.4565 ms]


COLLECT:
xap_p_lfmli/iter/1024   time:   [24.962 µs 25.118 µs 25.279 µs]
xap_p_lfmli/xap/1024    time:   [27.604 µs 27.945 µs 28.305 µs]

xap_p_lfmli/iter/32768  time:   [1.2495 ms 1.2625 ms 1.2778 ms]
xap_p_lfmli/xap/32768   time:   [1.3253 ms 1.3328 ms 1.3407 ms]


COLLECT BY LOOP:
xap_p_lfmli/iter/1024   time:   [25.290 µs 25.500 µs 25.727 µs]
xap_p_lfmli/xap/1024    time:   [32.855 µs 33.404 µs 34.004 µs]

xap_p_lfmli/iter/32768  time:   [1.1489 ms 1.1560 ms 1.1633 ms]
xap_p_lfmli/xap/32768   time:   [1.3652 ms 1.3759 ms 1.3876 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, xap_variants::Id};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = CollectByLoop;

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

fn f4(i: u64) -> impl IntoIterator<Item = u64> {
    [3 * i + 2, 2 * i + 5, i + 75]
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

    let mut group = c.benchmark_group("xap_p_lfmli");

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
