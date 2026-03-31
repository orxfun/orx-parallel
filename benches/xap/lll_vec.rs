/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_lll_vec/iter/1024   time:   [55.984 µs 56.500 µs 57.064 µs]
xap_lll_vec/xap/1024    time:   [139.55 µs 141.48 µs 143.61 µs]

xap_lll_vec/iter/32768  time:   [1.6580 ms 1.6721 ms 1.6874 ms]
xap_lll_vec/xap/32768   time:   [4.3515 ms 4.4125 ms 4.4801 ms]


SUM BY LOOP:
xap_lll_vec/iter/1024   time:   [475.55 µs 479.01 µs 482.62 µs]
xap_lll_vec/xap/1024    time:   [630.01 µs 641.24 µs 653.56 µs]

xap_lll_vec/iter/32768  time:   [15.154 ms 15.240 ms 15.328 ms]
xap_lll_vec/xap/32768   time:   [18.501 ms 18.609 ms 18.719 ms]


REDUCE:
xap_lll_vec/iter/1024   time:   [58.274 µs 59.197 µs 60.241 µs]
xap_lll_vec/xap/1024    time:   [138.50 µs 139.59 µs 140.82 µs]

xap_lll_vec/iter/32768  time:   [1.9062 ms 1.9176 ms 1.9294 ms]
xap_lll_vec/xap/32768   time:   [4.5249 ms 4.5824 ms 4.6443 ms]


COLLECT:
xap_lll_vec/iter/1024   time:   [595.58 µs 602.91 µs 610.31 µs]
xap_lll_vec/xap/1024    time:   [749.89 µs 757.35 µs 765.84 µs]

xap_lll_vec/iter/32768  time:   [30.157 ms 30.363 ms 30.571 ms]
xap_lll_vec/xap/32768   time:   [33.710 ms 34.065 ms 34.455 ms]


COLLECT BY LOOP:
xap_lll_vec/iter/1024   time:   [416.51 µs 418.30 µs 420.23 µs]
xap_lll_vec/xap/1024    time:   [604.21 µs 609.78 µs 615.32 µs]

xap_lll_vec/iter/32768  time:   [23.533 ms 23.791 ms 24.068 ms]
xap_lll_vec/xap/32768   time:   [31.880 ms 32.212 ms 32.551 ms]

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

fn f1(i: u64) -> Vec<u64> {
    vec![i + 1, i * 2, i + 5, i + 4, i, i.saturating_sub(3), 7 * i]
}

fn f2(i: u64) -> Vec<u64> {
    vec![i * 2 + 1, i, i.saturating_sub(7)]
}

fn f3(i: u64) -> Vec<u64> {
    vec![i / 3, i + 7, i.saturating_sub(4), i / 4, i]
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

    let mut group = c.benchmark_group("xap_lll_vec");

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
