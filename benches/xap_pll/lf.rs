/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_p_lfmli2/iter/1024  time:   [8.3859 µs 8.4390 µs 8.4985 µs]
xap_p_lfmli2/xap/1024   time:   [29.862 µs 30.046 µs 30.230 µs]

xap_p_lfmli2/iter/32768 time:   [399.88 µs 402.18 µs 404.62 µs]
xap_p_lfmli2/xap/32768  time:   [1.0680 ms 1.0761 ms 1.0848 ms


SUM BY LOOP:
xap_p_lfmli2/iter/1024  time:   [18.442 µs 18.567 µs 18.696 µs]
xap_p_lfmli2/xap/1024   time:   [21.340 µs 21.481 µs 21.630 µs]

xap_p_lfmli2/iter/32768 time:   [879.68 µs 883.55 µs 887.94 µs]
xap_p_lfmli2/xap/32768  time:   [889.73 µs 899.58 µs 909.90 µs]


REDUCE:
xap_p_lfmli2/iter/1024  time:   [7.7593 µs 7.8340 µs 7.9127 µs]
xap_p_lfmli2/xap/1024   time:   [25.231 µs 25.495 µs 25.767 µs]

xap_p_lfmli2/iter/32768 time:   [370.46 µs 372.34 µs 374.35 µs]
xap_p_lfmli2/xap/32768  time:   [899.34 µs 904.19 µs 909.55 µs]


COLLECT:
xap_p_lfmli2/iter/1024  time:   [36.430 µs 36.801 µs 37.174 µs]
xap_p_lfmli2/xap/1024   time:   [46.999 µs 47.428 µs 47.874 µs]

xap_p_lfmli2/iter/32768 time:   [1.3517 ms 1.3609 ms 1.3705 ms]
xap_p_lfmli2/xap/32768  time:   [1.5928 ms 1.6018 ms 1.6113 ms]


COLLECT BY LOOP:
xap_p_lfmli2/iter/1024  time:   [26.253 µs 26.500 µs 26.777 µs]
xap_p_lfmli2/xap/1024   time:   [26.930 µs 27.151 µs 27.370 µs]

xap_p_lfmli2/iter/32768 time:   [1.1327 ms 1.1429 ms 1.1529 ms]
xap_p_lfmli2/xap/32768  time:   [1.0614 ms 1.0670 ms 1.0732 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = Reduce;

trait Exp {
    type Out;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>;
}

pub struct Sum;
impl Exp for Sum {
    type Out = u64;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>,
    {
        let mut x = 0;
        for i in inputs {
            let i = black_box(i);
            x += fmap(i).into_iter().sum::<u64>();
        }
        x
    }
}

pub struct SumByLoop;
impl Exp for SumByLoop {
    type Out = u64;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>,
    {
        let mut x = 0;
        for i in inputs {
            let i = black_box(i);
            for j in fmap(i).into_iter() {
                x += j;
            }
        }
        x
    }
}

pub struct Reduce;
impl Exp for Reduce {
    type Out = Option<u64>;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>,
    {
        let mut x = None;
        for i in inputs {
            let i = black_box(i);
            if let Some(y) = fmap(i).into_iter().reduce(|x, y| 2 * x + y + 7) {
                x = match &mut x {
                    None => Some(y),
                    Some(x) => Some(2 * *x + y + 7),
                };
            }
        }
        x
    }
}

pub struct Collect;
impl Exp for Collect {
    type Out = Vec<u64>;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>,
    {
        let mut x = vec![];
        for i in inputs {
            let i = black_box(i);
            x.extend(fmap(i));
        }
        x
    }
}

pub struct CollectByLoop;
impl Exp for CollectByLoop {
    type Out = Vec<u64>;

    fn out<F, I>(inputs: impl Iterator<Item = u64>, fmap: F) -> Self::Out
    where
        F: Fn(u64) -> I,
        I: IntoIterator<Item = u64>,
    {
        let mut x = vec![];
        for i in inputs {
            let i = black_box(i);
            for j in fmap(i) {
                x.push(j);
            }
        }
        x
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

fn f3(i: u64) -> u64 {
    i * 3 + 5
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied();
    E::out(iter, |i| f1(i).into_iter().filter(f2))
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).filter(f2);
    let iter = inputs.iter().copied();
    E::out(iter, |i| xap.xap(i))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_p_lfmli2");

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
