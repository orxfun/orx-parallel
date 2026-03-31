/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_pll_l_vec/iter/1024 time:   [1.9226 µs 1.9392 µs 1.9575 µs]
xap_pll_l_vec/xap/1024  time:   [1.8065 µs 1.8210 µs 1.8363 µs]

xap_pll_l_vec/iter/32768
xap_pll_l_vec/xap/32768 time:   [57.489 µs 58.171 µs 58.866 µs]


SUM BY LOOP:
xap_pll_l_vec/iter/1024 time:   [1.3692 µs 1.3822 µs 1.3949 µs]
xap_pll_l_vec/xap/1024  time:   [1.3872 µs 1.3929 µs 1.3987 µs]

xap_pll_l_vec/iter/32768time:   [46.317 µs 46.761 µs 47.225 µs]
xap_pll_l_vec/xap/32768 time:   [47.148 µs 47.621 µs 48.115 µs]


REDUCE:
xap_pll_l_vec/iter/1024 time:   [1.7080 µs 1.7200 µs 1.7325 µs]
xap_pll_l_vec/xap/1024  time:   [1.7633 µs 1.7829 µs 1.8036 µs]

xap_pll_l_vec/iter/32768time:   [56.947 µs 57.450 µs 58.016 µs]
xap_pll_l_vec/xap/32768 time:   [56.925 µs 57.567 µs 58.264 µs]


COLLECT:
xap_pll_l_vec/iter/1024 time:   [17.616 µs 17.746 µs 17.880 µs]
xap_pll_l_vec/xap/1024  time:   [16.593 µs 16.711 µs 16.844 µs]

xap_pll_l_vec/iter/32768time:   [542.54 µs 546.78 µs 550.98 µs]
xap_pll_l_vec/xap/32768 time:   [579.86 µs 587.06 µs 595.03 µs]


COLLECT BY LOOP:
xap_pll_l_vec/iter/1024 time:   [19.292 µs 19.510 µs 19.716 µs]
xap_pll_l_vec/xap/1024  time:   [18.116 µs 18.277 µs 18.428 µs]

xap_pll_l_vec/iter/32768time:   [578.77 µs 583.68 µs 588.74 µs]
xap_pll_l_vec/xap/32768 time:   [618.36 µs 622.34 µs 626.42 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, xap_variants::Id};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = Collect;

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

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied();
    E::out(iter, f1)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1);
    let iter = inputs.iter().copied();
    E::out(iter, |i| xap.xap(i))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_p_l_vec");

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
