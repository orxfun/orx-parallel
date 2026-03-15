/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_p_lfmi/iter/1024    time:   [3.9065 µs 3.9385 µs 3.9744 µs]
xap_p_lfmi/xap/1024     time:   [5.8216 µs 5.8783 µs 5.9402 µs]

xap_p_lfmi/iter/32768   time:   [381.70 µs 384.56 µs 387.92 µs]
xap_p_lfmi/xap/32768    time:   [503.21 µs 508.23 µs 513.72 µs]


SUM BY LOOP:
xap_p_lfmi/iter/1024    time:   [5.8653 µs 5.9048 µs 5.9465 µs]
xap_p_lfmi/xap/1024     time:   [5.0922 µs 5.1234 µs 5.1554 µs]

xap_p_lfmi/iter/32768   time:   [493.89 µs 496.24 µs 498.50 µs]
xap_p_lfmi/xap/32768    time:   [487.27 µs 491.77 µs 496.54 µs]


REDUCE:
xap_p_lfmi/iter/1024    time:   [4.8923 µs 4.9419 µs 4.9943 µs]
xap_p_lfmi/xap/1024     time:   [5.7552 µs 5.8084 µs 5.8617 µs]

xap_p_lfmi/iter/32768   time:   [356.46 µs 360.64 µs 365.15 µs]
xap_p_lfmi/xap/32768    time:   [518.51 µs 525.08 µs 532.60 µs]


COLLECT:
xap_p_lfmi/iter/1024    time:   [7.3950 µs 7.4889 µs 7.5896 µs]
xap_p_lfmi/xap/1024     time:   [7.1151 µs 7.1778 µs 7.2418 µs]

xap_p_lfmi/iter/32768   time:   [643.43 µs 648.93 µs 655.21 µs]
xap_p_lfmi/xap/32768    time:   [572.61 µs 575.07 µs 577.94 µs]


COLLECT BY LOOP:
xap_p_lfmi/iter/1024    time:   [7.1747 µs 7.2488 µs 7.3220 µs]
xap_p_lfmi/xap/1024     time:   [8.8062 µs 8.8584 µs 8.9139 µs]

xap_p_lfmi/iter/32768   time:   [564.79 µs 570.79 µs 577.25 µs]
xap_p_lfmi/xap/32768    time:   [503.52 µs 508.85 µs 515.34 µs]

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

fn f1(i: u64) -> impl Iterator<Item = u64> {
    (1..7).map(move |x| 3 * x + i + 7)
}

fn f2(i: &u64) -> bool {
    !(i + 7).is_multiple_of(11)
}

fn f3(i: u64) -> u64 {
    i * 3 + 5
}

fn f4(i: u64) -> Option<u64> {
    match (i + 5).is_multiple_of(4) {
        true => None,
        false => Some(3 * i + 1),
    }
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied();
    E::out(iter, |i| {
        f1(i).into_iter().filter(f2).map(f3).filter_map(f4)
    })
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).filter(f2).map(f3).filter_map(f4);
    let iter = inputs.iter().copied();
    E::out(iter, |i| xap.xap(i))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_p_lfmi");

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
