/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ll_iter/iter/1024   time:   [2.7522 µs 2.7724 µs 2.7929 µs]
xap_ll_iter/xap/1024    time:   [3.0061 µs 3.0261 µs 3.0479 µs]

xap_ll_iter/iter/32768  time:   [233.29 µs 235.16 µs 237.01 µs]
xap_ll_iter/xap/32768   time:   [260.78 µs 262.77 µs 264.83 µs]


SUM BY LOOP:
xap_ll_iter/iter/1024   time:   [9.4253 µs 9.4819 µs 9.5413 µs]
xap_ll_iter/xap/1024    time:   [9.4084 µs 9.4734 µs 9.5386 µs]

xap_ll_iter/iter/32768  time:   [523.08 µs 526.67 µs 530.28 µs]
xap_ll_iter/xap/32768   time:   [547.53 µs 553.21 µs 559.77 µs]


REDUCE:
xap_ll_iter/iter/1024   time:   [2.4275 µs 2.4411 µs 2.4552 µs]
xap_ll_iter/xap/1024    time:   [2.4296 µs 2.4480 µs 2.4671 µs]

xap_ll_iter/iter/32768  time:   [223.44 µs 227.94 µs 232.36 µs]
xap_ll_iter/xap/32768   time:   [249.46 µs 252.17 µs 255.22 µs]


COLLECT:
xap_ll_iter/iter/1024   time:   [13.447 µs 13.573 µs 13.703 µs]
xap_ll_iter/xap/1024    time:   [15.782 µs 15.917 µs 16.066 µs]

xap_ll_iter/iter/32768  time:   [645.47 µs 649.60 µs 653.87 µs]
xap_ll_iter/xap/32768   time:   [727.58 µs 734.69 µs 742.37 µs]


COLLECT BY LOOP:
xap_ll_iter/iter/1024   time:   [18.372 µs 18.710 µs 19.059 µs]
xap_ll_iter/xap/1024    time:   [13.739 µs 13.938 µs 14.175 µs]

xap_ll_iter/iter/32768  time:   [754.31 µs 764.78 µs 776.33 µs]
xap_ll_iter/xap/32768   time:   [664.47 µs 671.31 µs 678.67 µs]

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
    let iter = inputs.flat_map(|x| xap.xap(x));
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
