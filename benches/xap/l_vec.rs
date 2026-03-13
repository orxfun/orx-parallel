/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_l_vec/xap/1024      time:   [669.22 ns 675.11 ns 681.49 ns]
xap_l_vec/iter/1024     time:   [784.28 ns 792.76 ns 802.53 ns]

xap_l_vec/xap/32768     time:   [22.664 µs 23.045 µs 23.452 µs]
xap_l_vec/iter/32768    time:   [22.512 µs 22.762 µs 23.091 µs]


SUM BY LOOP:
xap_l_vec/xap/1024      time:   [9.2864 µs 9.3914 µs 9.4973 µs]
xap_l_vec/iter/1024     time:   [9.3803 µs 9.4893 µs 9.5950 µs]

xap_l_vec/xap/32768     time:   [313.99 µs 320.22 µs 326.66 µs]
xap_l_vec/iter/32768    time:   [302.26 µs 305.50 µs 308.84 µs]


REDUCE:
xap_l_vec/xap/1024      time:   [1.9825 µs 2.0055 µs 2.0287 µs]
xap_l_vec/iter/1024     time:   [1.9883 µs 2.0086 µs 2.0299 µs]

xap_l_vec/xap/32768     time:   [56.707 µs 57.055 µs 57.439 µs]
xap_l_vec/iter/32768    time:   [66.996 µs 69.205 µs 71.633 µs]


COLLECT:
xap_l_vec/xap/1024      time:   [1.7788 µs 1.8124 µs 1.8465 µs]
xap_l_vec/iter/1024     time:   [1.8456 µs 1.8623 µs 1.8801 µs]

xap_l_vec/xap/32768     time:   [65.929 µs 66.925 µs 67.970 µs]
xap_l_vec/iter/32768    time:   [63.919 µs 64.767 µs 65.723 µs]


COLLECT BY LOOP:
xap_l_vec/xap/1024      time:   [29.076 µs 29.469 µs 29.940 µs]
xap_l_vec/iter/1024     time:   [30.606 µs 31.089 µs 31.634 µs]

xap_l_vec/xap/32768     time:   [969.54 µs 987.37 µs 1.0106 ms]
xap_l_vec/iter/32768    time:   [943.80 µs 955.13 µs 968.01 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = Reduce;

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

pub struct Reduce;
impl Exp for Reduce {
    type Out = Option<u64>;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        i.reduce(|x, y| 2 * x + y + 7)
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
    let iter = inputs.flat_map(|x| xap.xap(x));
    E::out(iter)
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_l_vec");

    for n in len {
        let input = inputs(n);
        let expected = iter::<Output>(&input);

        group.bench_with_input(BenchmarkId::new("xap", n), &n, |b, _| {
            assert_eq!(&expected, &xap::<Output>(&input));
            b.iter(|| xap::<Output>(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("iter", n), &n, |b, _| {
            assert_eq!(&expected, &iter::<Output>(&input));
            b.iter(|| iter::<Output>(black_box(&input)))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
