/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ll_iter/iter/1024   time:   [3.6240 µs 3.6659 µs 3.7091 µs]
xap_ll_iter/xap/1024    time:   [3.3205 µs 3.3631 µs 3.4080 µs]

xap_ll_iter/iter/32768  time:   [273.92 µs 276.06 µs 278.23 µs]
xap_ll_iter/xap/32768   time:   [256.85 µs 259.29 µs 261.92 µs]

SUM BY LOOP:
xap_ll_iter/iter/1024   time:   [13.064 µs 13.401 µs 13.831 µs]
xap_ll_iter/xap/1024    time:   [11.643 µs 11.802 µs 11.973 µs]

xap_ll_iter/iter/32768  time:   [612.47 µs 618.62 µs 625.35 µs]
xap_ll_iter/xap/32768   time:   [566.48 µs 570.56 µs 575.28 µs]


REDUCE:
xap_ll_iter/iter/1024   time:   [2.5962 µs 2.6270 µs 2.6575 µs]
xap_ll_iter/xap/1024    time:   [2.5394 µs 2.5640 µs 2.5903 µs]

xap_ll_iter/iter/32768  time:   [242.95 µs 248.40 µs 253.87 µs]
xap_ll_iter/xap/32768   time:   [242.87 µs 246.47 µs 250.64 µs]


COLLECT:
xap_ll_iter/iter/1024   time:   [36.164 µs 36.963 µs 37.791 µs]
xap_ll_iter/xap/1024    time:   [41.310 µs 42.143 µs 42.892 µs]

xap_ll_iter/iter/32768  time:   [846.29 µs 859.80 µs 875.81 µs]
xap_ll_iter/xap/32768   time:   [1.0330 ms 1.0439 ms 1.0564 ms]

COLLECT BY LOOP:
xap_ll_iter/iter/1024   time:   [25.497 µs 26.367 µs 27.209 µs]
xap_ll_iter/xap/1024    time:   [25.662 µs 25.930 µs 26.250 µs]

xap_ll_iter/iter/32768  time:   [857.39 µs 866.09 µs 876.15 µs]
xap_ll_iter/xap/32768   time:   [835.29 µs 842.20 µs 849.16 µs]

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

fn f1(i: u64) -> impl IntoIterator<Item = u64> {
    (2..5)
        .map(move |x| i + 2 * x as u64 + 5)
        .filter(|x| !x.is_multiple_of(3))
}

fn f2(i: u64) -> impl IntoIterator<Item = u64> {
    (6..8)
        .map(move |x| i + 5 * x as u64 + 2)
        .filter(|x| !x.is_multiple_of(3))
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

    let mut group = c.benchmark_group("xap_ll_iter");

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
