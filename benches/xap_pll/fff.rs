/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_p_fff/iter/1024     time:   [947.30 ns 954.75 ns 962.55 ns]
xap_p_fff/xap/1024      time:   [832.36 ns 843.12 ns 854.76 ns]

xap_p_fff/iter/32768    time:   [93.668 µs 95.092 µs 96.553 µs]
xap_p_fff/xap/32768     time:   [88.724 µs 89.564 µs 90.409 µs]


SUM BY LOOP:
xap_p_fff/iter/1024     time:   [819.36 ns 828.05 ns 837.22 ns]
xap_p_fff/xap/1024      time:   [973.43 ns 983.63 ns 994.93 ns]

xap_p_fff/iter/32768    time:   [105.19 µs 106.10 µs 107.01 µs]
xap_p_fff/xap/32768     time:   [110.16 µs 111.45 µs 112.78 µs]


REDUCE:
xap_p_fff/iter/1024     time:   [1.0343 µs 1.0452 µs 1.0570 µs]
xap_p_fff/xap/1024      time:   [1.0774 µs 1.0900 µs 1.1028 µs]

xap_p_fff/iter/32768    time:   [124.14 µs 125.24 µs 126.46 µs]
xap_p_fff/xap/32768     time:   [125.58 µs 126.58 µs 127.70 µs]


COLLECT:
xap_p_fff/iter/1024     time:   [1.5057 µs 1.5172 µs 1.5290 µs]
xap_p_fff/xap/1024      time:   [1.4997 µs 1.5095 µs 1.5202 µs]

xap_p_fff/iter/32768    time:   [169.38 µs 170.99 µs 172.61 µs]
xap_p_fff/xap/32768     time:   [135.96 µs 138.02 µs 140.25 µs]


COLLECT BY LOOP:
xap_p_fff/iter/1024     time:   [1.5113 µs 1.5246 µs 1.5388 µs]
xap_p_fff/xap/1024      time:   [1.5871 µs 1.5957 µs 1.6055 µs]

xap_p_fff/iter/32768    time:   [152.53 µs 154.09 µs 155.88 µs]
xap_p_fff/xap/32768     time:   [151.48 µs 152.81 µs 154.22 µs]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, xap_variants::Id};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = Collect;

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

fn f1(i: &u64) -> bool {
    !i.is_multiple_of(3)
}

fn f2(i: &u64) -> bool {
    !(i + 7).is_multiple_of(11)
}

fn f3(i: &u64) -> bool {
    !(i + 11).is_multiple_of(5)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied();
    E::out(iter, |i| (f1(&i) && f2(&i) && f3(&i)).then_some(i))
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter(f1).filter(f2).filter(f3);
    let iter = inputs.iter().copied();
    E::out(iter, |i| xap.xap(i))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_p_fff");

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
