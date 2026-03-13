/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_l_vec/iter/1024     time:   [665.94 ns 673.66 ns 681.57 ns]
xap_l_vec/xap/1024      time:   [710.17 ns 717.87 ns 725.97 ns]

xap_l_vec/iter/32768    time:   [24.389 µs 24.839 µs 25.319 µs]
xap_l_vec/xap/32768     time:   [26.424 µs 26.995 µs 27.637 µs]

SUM BY LOOP:
xap_l_vec/iter/1024     time:   [10.157 µs 10.457 µs 10.775 µs]
xap_l_vec/xap/1024      time:   [8.7531 µs 8.8391 µs 8.9233 µs]

xap_l_vec/iter/32768    time:   [279.71 µs 287.67 µs 295.63 µs]
xap_l_vec/xap/32768     time:   [272.73 µs 275.83 µs 278.85 µs]


COLLECT:
xap_l_vec/iter/1024     time:   [1.8171 µs 1.8577 µs 1.8990 µs]
xap_l_vec/xap/1024      time:   [2.4894 µs 2.5683 µs 2.6432 µs]

xap_l_vec/iter/32768    time:   [68.815 µs 70.039 µs 71.282 µs]
xap_l_vec/xap/32768     time:   [101.06 µs 105.06 µs 108.96 µs]

COLLECT BY LOOP:
xap_l_vec/iter/1024     time:   [27.841 µs 28.122 µs 28.432 µs]
xap_l_vec/xap/1024      time:   [28.073 µs 28.453 µs 28.844 µs]

xap_l_vec/iter/32768    time:   [929.14 µs 939.05 µs 949.36 µs]
xap_l_vec/xap/32768     time:   [916.93 µs 925.00 µs 933.50 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = Sum;

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

fn f1(i: u64) -> [u64; 7] {
    [i + 1, i * 2, i + 5, i + 4, i, i.saturating_sub(3), 7 * i]
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1);
    let inputs = inputs.iter().copied();
    let iter = XapIter::new(inputs, xap);
    E::out(iter)
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_l_vec");

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
