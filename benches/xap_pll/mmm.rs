/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_p_mmm/iter/1024     time:   [709.15 ns 713.40 ns 718.01 ns]
xap_p_mmm/xap/1024      time:   [723.15 ns 727.95 ns 733.21 ns]

xap_p_mmm/iter/32768    time:   [23.214 µs 23.347 µs 23.491 µs]
xap_p_mmm/xap/32768     time:   [23.265 µs 23.431 µs 23.621 µs]


SUM BY LOOP:
xap_p_mmm/iter/1024     time:   [793.04 ns 799.34 ns 806.06 ns]
xap_p_mmm/xap/1024      time:   [829.07 ns 834.56 ns 839.90 ns]

xap_p_mmm/iter/32768    time:   [26.337 µs 26.493 µs 26.671 µs]
xap_p_mmm/xap/32768     time:   [26.602 µs 26.826 µs 27.082 µs]


REDUCE:
xap_p_mmm/iter/1024     time:   [668.77 ns 673.83 ns 678.76 ns]
xap_p_mmm/xap/1024      time:   [684.81 ns 690.92 ns 696.84 ns]

xap_p_mmm/iter/32768    time:   [21.915 µs 22.089 µs 22.282 µs]
xap_p_mmm/xap/32768     time:   [21.933 µs 22.050 µs 22.174 µs]


COLLECT:
xap_p_mmm/iter/1024     time:   [999.03 ns 1.0061 µs 1.0132 µs]
xap_p_mmm/xap/1024      time:   [1.2049 µs 1.2116 µs 1.2186 µs]

xap_p_mmm/iter/32768    time:   [33.629 µs 34.380 µs 35.160 µs]
xap_p_mmm/xap/32768     time:   [32.820 µs 33.508 µs 34.212 µs]


COLLECT BY LOOP:
xap_p_mmm/iter/1024     time:   [1.0957 µs 1.1011 µs 1.1067 µs]
xap_p_mmm/xap/1024      time:   [1.4515 µs 1.4630 µs 1.4742 µs]

xap_p_mmm/iter/32768    time:   [48.216 µs 48.749 µs 49.238 µs]
xap_p_mmm/xap/32768     time:   [47.378 µs 47.938 µs 48.577 µs]

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

fn f1(i: u64) -> u64 {
    2 * i + 1
}

fn f2(i: u64) -> u64 {
    (7 * i).saturating_sub(71)
}

fn f3(i: u64) -> u64 {
    i * 3 + 5
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied();
    E::out(iter, |i| [f3(f2(f1(i)))])
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().map(f1).map(f2).map(f3);
    let iter = inputs.iter().copied();
    E::out(iter, |i| xap.xap(i))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_p_mmm");

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
