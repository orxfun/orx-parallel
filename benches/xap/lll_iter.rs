/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_lll_iter/iter/1024  time:   [10.484 µs 10.667 µs 10.858 µs]
xap_lll_iter/xap/1024   time:   [10.269 µs 10.482 µs 10.705 µs]

xap_lll_iter/iter/32768 time:   [337.67 µs 343.98 µs 350.83 µs]
xap_lll_iter/xap/32768  time:   [345.16 µs 351.74 µs 358.69 µs]

SUM BY LOOP:
xap_lll_iter/iter/1024  time:   [65.246 µs 66.115 µs 67.028 µs]
xap_lll_iter/xap/1024   time:   [60.884 µs 61.581 µs 62.313 µs]

xap_lll_iter/iter/32768 time:   [1.9315 ms 1.9506 ms 1.9704 ms]
xap_lll_iter/xap/32768  time:   [1.8352 ms 1.8525 ms 1.8707 ms]


REDUCE:
xap_lll_iter/iter/1024  time:   [29.775 µs 30.122 µs 30.525 µs]
xap_lll_iter/xap/1024   time:   [29.617 µs 29.897 µs 30.193 µs]

xap_lll_iter/iter/32768 time:   [945.33 µs 957.35 µs 970.59 µs]
xap_lll_iter/xap/32768  time:   [956.26 µs 964.57 µs 973.59 µs]


COLLECT:
xap_lll_iter/iter/1024  time:   [74.686 µs 75.528 µs 76.491 µs]
xap_lll_iter/xap/1024   time:   [92.915 µs 94.745 µs 96.674 µs]

xap_lll_iter/iter/32768 time:   [2.3660 ms 2.3893 ms 2.4143 ms]
xap_lll_iter/xap/32768  time:   [2.8669 ms 2.8932 ms 2.9210 ms]

COLLECT BY LOOP:
xap_lll_iter/iter/1024  time:   [71.586 µs 73.142 µs 74.735 µs]
xap_lll_iter/xap/1024   time:   [71.181 µs 73.001 µs 75.035 µs]

xap_lll_iter/iter/32768 time:   [2.2528 ms 2.2728 ms 2.2958 ms]
xap_lll_iter/xap/32768  time:   [2.5069 ms 2.5482 ms 2.5897 ms]

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
        .filter(|x| !x.is_multiple_of(999))
}

fn f2(i: u64) -> impl IntoIterator<Item = u64> {
    (6..8)
        .map(move |x| i + 5 * x as u64 + 2)
        .filter(|x| !x.is_multiple_of(999))
}

fn f3(i: u64) -> impl IntoIterator<Item = u64> {
    (9..12)
        .map(move |x| i + 3 * x as u64 + 7)
        .filter(|x| !x.is_multiple_of(999))
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

    let mut group = c.benchmark_group("xap_lll_iter");

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
