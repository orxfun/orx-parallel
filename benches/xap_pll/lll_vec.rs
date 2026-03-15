/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_pll_lll_vec/iter/1024   time:   [165.45 µs 168.11 µs 171.03 µs]
xap_pll_lll_vec/xap/1024    time:   [164.20 µs 166.61 µs 169.26 µs]

xap_pll_lll_vec/iter/32768  time:   [5.3006 ms 5.3824 ms 5.4652 ms]
xap_pll_lll_vec/xap/32768   time:   [5.3622 ms 5.4463 ms 5.5326 ms]


SUM BY LOOP:
xap_pll_lll_vec/iter/1024   time:   [505.63 µs 509.94 µs 514.58 µs]
xap_pll_lll_vec/xap/1024    time:   [524.16 µs 529.58 µs 535.11 µs]

xap_pll_lll_vec/iter/32768  time:   [17.801 ms 18.024 ms 18.255 ms]
xap_pll_lll_vec/xap/32768   time:   [18.480 ms 18.768 ms 19.071 ms]


REDUCE:
xap_pll_lll_vec/iter/1024   time:   [178.02 µs 180.25 µs 182.72 µs]
xap_pll_lll_vec/xap/1024    time:   [145.05 µs 146.42 µs 148.03 µs]

xap_pll_lll_vec/iter/32768  time:   [5.2488 ms 5.2990 ms 5.3505 ms]
xap_pll_lll_vec/xap/32768   time:   [4.8106 ms 4.8778 ms 4.9490 ms]


COLLECT:
xap_pll_lll_vec/iter/1024   time:   [698.43 µs 710.19 µs 721.66 µs]
xap_pll_lll_vec/xap/1024    time:   [611.88 µs 618.48 µs 625.38 µs]

xap_pll_lll_vec/iter/32768  time:   [32.754 ms 33.063 ms 33.378 ms]
xap_pll_lll_vec/xap/32768   time:   [31.115 ms 31.456 ms 31.807 ms]


COLLECT BY LOOP:
xap_pll_lll_vec/iter/1024   time:   [644.20 µs 655.77 µs 668.02 µs]
xap_pll_lll_vec/xap/1024    time:   [652.98 µs 670.70 µs 688.54 µs]

xap_pll_lll_vec/iter/32768  time:   [34.021 ms 34.558 ms 35.111 ms]
xap_pll_lll_vec/xap/32768   time:   [30.363 ms 30.871 ms 31.400 ms]

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

fn f3(i: u64) -> Vec<u64> {
    vec![i / 3, i + 7, i.saturating_sub(4), i / 4, i]
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied();
    E::out(iter, |i| f1(i).into_iter().flat_map(f2).flat_map(f3))
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).flat_map(f2).flat_map(f3);
    let iter = inputs.iter().copied();
    E::out(iter, |i| xap.xap(i))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_p_lll_vec");

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
