/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_lll_cons/iter/1024  time:   [42.997 µs 43.594 µs 44.276 µs]
xap_lll_cons/xap/1024   time:   [24.837 µs 25.160 µs 25.526 µs]

xap_lll_cons/iter/32768 time:   [1.3582 ms 1.3830 ms 1.4111 ms]
xap_lll_cons/xap/32768  time:   [836.88 µs 847.12 µs 856.96 µs]


SUM BY LOOP:
xap_lll_cons/iter/1024  time:   [80.073 µs 80.897 µs 81.752 µs]
xap_lll_cons/xap/1024   time:   [137.53 µs 138.60 µs 139.79 µs]

xap_lll_cons/iter/32768 time:   [2.6051 ms 2.6338 ms 2.6633 ms]
xap_lll_cons/xap/32768  time:   [4.6230 ms 4.6913 ms 4.7674 ms]


REDUCE:
xap_lll_cons/iter/1024  time:   [107.20 ns 108.08 ns 109.07 ns]
xap_lll_cons/xap/1024   time:   [92.274 ns 92.790 ns 93.350 ns]

xap_lll_cons/iter/32768 time:   [766.08 ns 790.75 ns 817.13 ns]
xap_lll_cons/xap/32768  time:   [819.38 ns 843.67 ns 867.46 ns]


COLLECT:
xap_lll_cons/iter/1024  time:   [31.272 µs 31.636 µs 31.972 µs]
xap_lll_cons/xap/1024   time:   [278.54 µs 281.08 µs 283.74 µs]

xap_lll_cons/iter/32768 time:   [1.4843 ms 1.5635 ms 1.6465 ms]
xap_lll_cons/xap/32768  time:   [22.806 ms 23.104 ms 23.414 ms]


COLLECT BY LOOP:
xap_lll_cons/iter/1024  time:   [214.34 µs 217.80 µs 221.47 µs]
xap_lll_cons/xap/1024   time:   [231.35 µs 235.54 µs 239.76 µs]

xap_lll_cons/iter/32768 time:   [19.276 ms 19.543 ms 19.841 ms]
xap_lll_cons/xap/32768  time:   [17.030 ms 17.171 ms 17.317 ms]

TODO: room for performance improvement with Collect

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, xap_variants::Id};
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

fn f2(i: u64) -> [u64; 3] {
    [i * 2 + 1, i, i.saturating_sub(7)]
}

fn f3(i: u64) -> [u64; 5] {
    [i / 3, i + 7, i.saturating_sub(4), i / 4, i]
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
    let inputs = inputs.iter().copied();
    let iter = inputs.flat_map(|x| xap.xap(x));
    E::out(iter)
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_lll_cons");

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
