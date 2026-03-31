/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_lf/iter/1024        time:   [2.4254 µs 2.4484 µs 2.4733 µs]
xap_lf/xap/1024         time:   [2.5140 µs 2.5719 µs 2.6408 µs]

xap_lf/iter/32768       time:   [213.78 µs 216.08 µs 218.51 µs]
xap_lf/xap/32768        time:   [218.37 µs 220.99 µs 223.80 µs]


SUM BY LOOP:
xap_lf/iter/1024        time:   [4.1965 µs 4.2240 µs 4.2516 µs]
xap_lf/xap/1024         time:   [6.0140 µs 6.0442 µs 6.0752 µs]

xap_lf/iter/32768       time:   [263.78 µs 265.06 µs 266.34 µs]
xap_lf/xap/32768        time:   [309.52 µs 311.11 µs 312.64 µs]


REDUCE:
xap_lf/iter/1024        time:   [4.1243 µs 4.1532 µs 4.1843 µs]
xap_lf/xap/1024         time:   [4.7423 µs 4.8119 µs 4.8752 µs]

xap_lf/iter/32768       time:   [145.46 µs 148.31 µs 151.51 µs]
xap_lf/xap/32768        time:   [262.48 µs 263.80 µs 265.17 µs]


COLLECT:
xap_lf/iter/1024        time:   [9.3105 µs 9.3820 µs 9.4608 µs]
xap_lf/xap/1024         time:   [9.4264 µs 9.5218 µs 9.6190 µs]

xap_lf/iter/32768       time:   [443.36 µs 446.89 µs 450.51 µs]
xap_lf/xap/32768        time:   [434.55 µs 437.39 µs 440.32 µs]


COLLECT BY LOOP:
xap_lf/iter/1024        time:   [14.146 µs 14.282 µs 14.428 µs]
xap_lf/xap/1024         time:   [10.072 µs 10.154 µs 10.249 µs]

xap_lf/iter/32768       time:   [625.70 µs 629.32 µs 633.11 µs]
xap_lf/xap/32768        time:   [549.41 µs 553.74 µs 558.66 µs]

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

fn f1(i: u64) -> impl Iterator<Item = u64> {
    (1..7).map(move |x| 3 * x + i + 7)
}

fn f2(i: &u64) -> bool {
    !(i + 7).is_multiple_of(11)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1).filter(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).filter(f2);
    let inputs = inputs.iter().copied();
    let iter = inputs.flat_map(|x| xap.xap(x));
    E::out(iter)
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_lf");

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
