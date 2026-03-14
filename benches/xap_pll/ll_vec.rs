/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_pll_ll_vec/iter/1024    time:   [30.352 µs 30.936 µs 31.528 µs]
xap_pll_ll_vec/xap/1024     time:   [30.413 µs 30.870 µs 31.363 µs]

xap_pll_ll_vec/iter/32768   time:   [981.13 µs 997.79 µs 1.0156 ms]
xap_pll_ll_vec/xap/32768    time:   [1.0089 ms 1.0251 ms 1.0425 ms]


SUM BY LOOP:
xap_pll_ll_vec/iter/1024    time:   [137.95 µs 139.87 µs 142.12 µs]
xap_pll_ll_vec/xap/1024     time:   [144.95 µs 146.21 µs 147.60 µs]

xap_pll_ll_vec/iter/32768   time:   [4.3268 ms 4.3639 ms 4.4006 ms]
xap_pll_ll_vec/xap/32768    time:   [4.3839 ms 4.4354 ms 4.4883 ms]


REDUCE:
xap_pll_ll_vec/iter/1024    time:   [25.079 µs 25.319 µs 25.580 µs]
xap_pll_ll_vec/xap/1024     time:   [26.129 µs 26.352 µs 26.594 µs]

xap_pll_ll_vec/iter/32768   time:   [765.44 µs 772.82 µs 780.56 µs]
xap_pll_ll_vec/xap/32768    time:   [857.76 µs 865.93 µs 875.05 µs]


COLLECT:
xap_pll_ll_vec/iter/1024    time:   [157.36 µs 159.06 µs 160.85 µs]
xap_pll_ll_vec/xap/1024     time:   [161.07 µs 162.78 µs 164.62 µs]

xap_pll_ll_vec/iter/32768   time:   [4.8489 ms 4.8843 ms 4.9217 ms]
xap_pll_ll_vec/xap/32768    time:   [5.2631 ms 5.3293 ms 5.3985 ms]


COLLECT BY LOOP:
xap_pll_ll_vec/iter/1024    time:   [136.55 µs 138.26 µs 140.04 µs]
xap_pll_ll_vec/xap/1024     time:   [139.45 µs 140.70 µs 142.06 µs]

xap_pll_ll_vec/iter/32768   time:   [4.2495 ms 4.2900 ms 4.3320 ms]
xap_pll_ll_vec/xap/32768    time:   [4.8304 ms 4.8935 ms 4.9600 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
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

fn f1(i: u64) -> Vec<u64> {
    vec![i + 1, i * 2, i + 5, i + 4, i, i.saturating_sub(3), 7 * i]
}

fn f2(i: u64) -> Vec<u64> {
    vec![i * 2 + 1, i, i.saturating_sub(7)]
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied();
    E::out(iter, |i| f1(i).into_iter().flat_map(f2))
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).flat_map(f2);
    let iter = inputs.iter().copied();
    E::out(iter, |i| xap.xap(i))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_pll_ll_vec");

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
