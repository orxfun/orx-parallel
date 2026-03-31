/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ll_cons/iter/1024   time:   [4.4150 µs 4.4668 µs 4.5218 µs]
xap_ll_cons/xap/1024    time:   [4.2309 µs 4.2770 µs 4.3270 µs]

xap_ll_cons/iter/32768  time:   [158.08 µs 162.10 µs 166.04 µs]
xap_ll_cons/xap/32768   time:   [134.15 µs 135.61 µs 137.27 µs]


SUM BY LOOP:
xap_ll_cons/iter/1024   time:   [14.037 µs 14.185 µs 14.345 µs]
xap_ll_cons/xap/1024    time:   [28.782 µs 29.395 µs 30.110 µs]

xap_ll_cons/iter/32768  time:   [456.53 µs 461.59 µs 467.25 µs]
xap_ll_cons/xap/32768   time:   [871.06 µs 887.23 µs 904.41 µs]


REDUCE:
xap_ll_cons/iter/1024   time:   [6.1495 µs 6.3426 µs 6.5225 µs]
xap_ll_cons/xap/1024    time:   [4.8069 µs 4.9385 µs 5.0648 µs]

xap_ll_cons/iter/32768  time:   [228.58 µs 233.85 µs 239.09 µs]
xap_ll_cons/xap/32768   time:   [137.70 µs 139.25 µs 140.94 µs]


COLLECT:
xap_ll_cons/iter/1024   time:   [4.7470 µs 4.8190 µs 4.9029 µs]
xap_ll_cons/xap/1024    time:   [58.646 µs 59.042 µs 59.457 µs]

xap_ll_cons/iter/32768  time:   [206.59 µs 208.08 µs 209.51 µs]
xap_ll_cons/xap/32768   time:   [1.6651 ms 1.6875 ms 1.7101 ms]


COLLECT BY LOOP:
xap_ll_cons/iter/1024   time:   [45.123 µs 45.555 µs 46.011 µs]
xap_ll_cons/xap/1024    time:   [32.275 µs 32.879 µs 33.603 µs]

xap_ll_cons/iter/32768  time:   [1.3377 ms 1.3528 ms 1.3704 ms]
xap_ll_cons/xap/32768   time:   [1.0510 ms 1.0838 ms 1.1164 ms]

TODO: room for performance improvement with Collect and SumByLoop

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, xap_variants::Id};
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

fn f2(i: u64) -> [u64; 3] {
    [i * 2 + 1, i, i.saturating_sub(7)]
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1).flat_map(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).flat_map(f2);
    let inputs = inputs.iter().copied();
    let iter = inputs.flat_map(|x| xap.xap(x));
    E::out(iter)
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_ll_cons");

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
