/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ll_vec/iter/1024    time:   [23.115 µs 23.344 µs 23.618 µs]
xap_ll_vec/xap/1024     time:   [128.41 µs 131.24 µs 134.09 µs]

xap_ll_vec/iter/32768   time:   [853.54 µs 872.33 µs 892.96 µs]
xap_ll_vec/xap/32768    time:   [4.9680 ms 5.0549 ms 5.1430 ms]

SUM BY LOOP:
xap_ll_vec/iter/1024    time:   [128.43 µs 129.85 µs 131.39 µs]
xap_ll_vec/xap/1024     time:   [142.45 µs 146.24 µs 149.91 µs]

xap_ll_vec/iter/32768   time:   [5.1991 ms 5.2810 ms 5.3641 ms]
xap_ll_vec/xap/32768    time:   [5.3174 ms 5.4091 ms 5.5007 ms]


COLLECT:
xap_ll_vec/iter/1024    time:   [159.55 µs 161.51 µs 163.92 µs]
xap_ll_vec/xap/1024     time:   [209.25 µs 213.36 µs 218.10 µs]

xap_ll_vec/iter/32768   time:   [5.0556 ms 5.1402 ms 5.2353 ms]
xap_ll_vec/xap/32768    time:   [7.2486 ms 7.3191 ms 7.3962 ms]

COLLECT BY LOOP:
xap_ll_vec/iter/1024    time:   [190.11 µs 193.02 µs 195.89 µs]
xap_ll_vec/xap/1024     time:   [180.79 µs 183.32 µs 186.20 µs]

xap_ll_vec/iter/32768   time:   [4.8297 ms 4.8923 ms 4.9568 ms]
xap_ll_vec/xap/32768    time:   [4.5771 ms 4.6155 ms 4.6560 ms]

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

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1).flat_map(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).flat_map(f2);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_ll_vec");

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
