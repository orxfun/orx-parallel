/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_lll_vec/iter/1024   time:   [66.218 µs 66.765 µs 67.339 µs]
xap_lll_vec/xap/1024    time:   [553.69 µs 560.21 µs 567.04 µs]

xap_lll_vec/iter/32768  time:   [2.0986 ms 2.1289 ms 2.1608 ms]
xap_lll_vec/xap/32768   time:   [15.934 ms 16.147 ms 16.380 ms]

SUM BY LOOP:
xap_lll_vec/iter/1024   time:   [536.43 µs 546.39 µs 557.27 µs]
xap_lll_vec/xap/1024    time:   [653.86 µs 662.99 µs 672.75 µs]

xap_lll_vec/iter/32768  time:   [31.777 ms 32.507 ms 33.284 ms]
xap_lll_vec/xap/32768   time:   [30.594 ms 31.221 ms 31.937 ms]


COLLECT:
xap_lll_vec/iter/1024   time:   [698.79 µs 708.78 µs 720.13 µs]
xap_lll_vec/xap/1024    time:   [810.91 µs 825.11 µs 841.69 µs]

xap_lll_vec/iter/32768  time:   [36.379 ms 36.832 ms 37.294 ms]
xap_lll_vec/xap/32768   time:   [37.453 ms 38.067 ms 38.710 ms]

COLLECT BY LOOP:
xap_lll_vec/iter/1024   time:   [573.10 µs 578.32 µs 584.05 µs]
xap_lll_vec/xap/1024    time:   [784.30 µs 795.98 µs 808.20 µs]

xap_lll_vec/iter/32768  time:   [26.849 ms 27.192 ms 27.567 ms]
xap_lll_vec/xap/32768   time:   [32.795 ms 33.234 ms 33.723 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
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
    let iter = inputs
        .iter()
        .copied()
        .flat_map(f1)
        .flat_map(f2)
        .flat_map(f3);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).flat_map(f2).flat_map(f3);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_lll_vec");

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
