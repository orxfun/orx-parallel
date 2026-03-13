/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_l_iter/iter/1024    time:   [1.4706 µs 1.4976 µs 1.5296 µs]
xap_l_iter/xap/1024     time:   [1.4438 µs 1.4604 µs 1.4799 µs]

xap_l_iter/iter/32768   time:   [42.793 µs 43.407 µs 44.071 µs]
xap_l_iter/xap/32768    time:   [42.738 µs 43.210 µs 43.723 µs]


SUM BY LOOP:
xap_l_iter/iter/1024    time:   [5.7771 µs 5.8468 µs 5.9155 µs]
xap_l_iter/xap/1024     time:   [5.6231 µs 5.6665 µs 5.7133 µs]

xap_l_iter/iter/32768   time:   [181.34 µs 182.46 µs 183.66 µs]
xap_l_iter/xap/32768    time:   [192.43 µs 194.38 µs 196.65 µs]


REDUCE:
xap_l_iter/iter/1024    time:   [2.6182 µs 2.6486 µs 2.6812 µs]
xap_l_iter/xap/1024     time:   [2.6489 µs 2.6733 µs 2.6993 µs]

xap_l_iter/iter/32768   time:   [80.274 µs 80.645 µs 81.055 µs]
xap_l_iter/xap/32768    time:   [80.161 µs 80.534 µs 80.931 µs]


COLLECT:
xap_l_iter/iter/1024    time:   [8.0966 µs 8.2066 µs 8.3258 µs]
xap_l_iter/xap/1024     time:   [7.9248 µs 8.0200 µs 8.1168 µs]

xap_l_iter/iter/32768   time:   [261.77 µs 264.78 µs 268.08 µs]
xap_l_iter/xap/32768    time:   [246.96 µs 248.79 µs 250.66 µs]


COLLECT BY LOOP:
xap_l_iter/iter/1024    time:   [7.4594 µs 7.5583 µs 7.6615 µs]
xap_l_iter/xap/1024     time:   [7.5535 µs 7.6090 µs 7.6694 µs]

xap_l_iter/iter/32768   time:   [252.46 µs 255.26 µs 258.47 µs]
xap_l_iter/xap/32768    time:   [239.74 µs 241.23 µs 242.77 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = SumByLoop;

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

    let mut group = c.benchmark_group("xap_l_iter");

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
