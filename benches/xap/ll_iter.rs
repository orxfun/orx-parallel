/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ll_iter/iter/1024   time:   [2.5507 µs 2.5706 µs 2.5915 µs]
xap_ll_iter/xap/1024    time:   [2.6102 µs 2.6366 µs 2.6630 µs]

xap_ll_iter/iter/32768  time:   [220.88 µs 223.05 µs 225.30 µs]
xap_ll_iter/xap/32768   time:   [228.26 µs 230.34 µs 232.61 µs]


SUM BY LOOP:
xap_ll_iter/iter/1024   time:   [9.7426 µs 9.8184 µs 9.8999 µs]
xap_ll_iter/xap/1024    time:   [8.8252 µs 8.8784 µs 8.9353 µs]

xap_ll_iter/iter/32768  time:   [494.22 µs 498.12 µs 502.64 µs]
xap_ll_iter/xap/32768   time:   [496.70 µs 501.00 µs 505.77 µs]


REDUCE:
xap_ll_iter/iter/1024   time:   [2.0172 µs 2.0307 µs 2.0449 µs]
xap_ll_iter/xap/1024    time:   [2.1070 µs 2.1244 µs 2.1438 µs]

xap_ll_iter/iter/32768  time:   [195.30 µs 196.75 µs 198.33 µs]
xap_ll_iter/xap/32768   time:   [211.48 µs 213.39 µs 215.30 µs]


COLLECT:
xap_ll_iter/iter/1024   time:   [11.599 µs 11.688 µs 11.779 µs]
xap_ll_iter/xap/1024    time:   [11.995 µs 12.040 µs 12.086 µs]

xap_ll_iter/iter/32768  time:   [599.54 µs 605.78 µs 612.57 µs]
xap_ll_iter/xap/32768   time:   [604.90 µs 611.03 µs 617.78 µs]


COLLECT BY LOOP:
xap_ll_iter/iter/1024   time:   [10.753 µs 10.817 µs 10.882 µs]
xap_ll_iter/xap/1024    time:   [10.658 µs 10.755 µs 10.849 µs]

xap_ll_iter/iter/32768  time:   [572.74 µs 576.96 µs 581.35 µs]
xap_ll_iter/xap/32768   time:   [571.12 µs 576.57 µs 582.13 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap, xap_iter::XapIter};
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
    let inputs = inputs.iter().copied();
    let iter = XapIter::new(inputs, xap);
    E::out(iter)
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
