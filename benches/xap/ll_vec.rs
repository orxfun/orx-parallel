/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ll_vec/iter/1024    time:   [30.407 µs 30.899 µs 31.416 µs]
xap_ll_vec/xap/1024     time:   [29.957 µs 30.381 µs 30.856 µs]

xap_ll_vec/iter/32768   time:   [1.0112 ms 1.0296 ms 1.0467 ms]
xap_ll_vec/xap/32768    time:   [926.67 µs 942.16 µs 957.44 µs]


SUM BY LOOP:
xap_ll_vec/iter/1024    time:   [153.17 µs 155.77 µs 158.58 µs]
xap_ll_vec/xap/1024     time:   [172.95 µs 175.48 µs 178.04 µs]

xap_ll_vec/iter/32768   time:   [4.9478 ms 5.0152 ms 5.0842 ms]
xap_ll_vec/xap/32768    time:   [5.2498 ms 5.3198 ms 5.3901 ms]


REDUCE:
xap_ll_vec/iter/1024    time:   [29.497 µs 29.971 µs 30.473 µs]
xap_ll_vec/xap/1024     time:   [27.997 µs 28.417 µs 28.873 µs]

xap_ll_vec/iter/32768   time:   [994.83 µs 1.0099 ms 1.0257 ms]
xap_ll_vec/xap/32768    time:   [986.25 µs 1.0010 ms 1.0155 ms]


COLLECT:
xap_ll_vec/iter/1024    time:   [207.57 µs 210.79 µs 213.92 µs]
xap_ll_vec/xap/1024     time:   [244.40 µs 248.10 µs 251.87 µs]

xap_ll_vec/iter/32768   time:   [6.1079 ms 6.1712 ms 6.2375 ms]
xap_ll_vec/xap/32768    time:   [8.1850 ms 8.2954 ms 8.4080 ms]


COLLECT BY LOOP:
xap_ll_vec/iter/1024    time:   [156.17 µs 158.33 µs 160.57 µs]
xap_ll_vec/xap/1024     time:   [144.44 µs 146.65 µs 148.89 µs]

xap_ll_vec/iter/32768   time:   [5.2283 ms 5.3042 ms 5.3841 ms]
xap_ll_vec/xap/32768    time:   [4.2370 ms 4.2973 ms 4.3604 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap, count::iter::FlatMapIterMany, fun::flat_map::FnFlatMap};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

type Output = Collect;

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

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1).flat_map(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).flat_map(f2);
    let iter = inputs.iter().copied();
    let iter = xap.into_iter_over(iter).into_iter();
    // let inputs = inputs.iter().copied();
    // let iter = inputs.flat_map(|x| xap.xap(x));
    E::out(iter)
}

fn xap_solo<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied();
    let iter = FlatMapIterMany::new(iter, FnFlatMap::new(f1));
    let iter = FlatMapIterMany::new(iter, FnFlatMap::new(f2));
    E::out(iter)
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];

    let mut group = c.benchmark_group("xap_ll_vec");

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

        group.bench_with_input(BenchmarkId::new("xap_solo", n), &n, |b, _| {
            assert_eq!(&expected, &xap_solo::<Output>(&input));
            b.iter(|| xap_solo::<Output>(black_box(&input)))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
