/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_i/iter/1024         time:   [552.98 ns 558.96 ns 565.85 ns]
xap_i/xap/1024          time:   [533.92 ns 538.51 ns 543.12 ns]

xap_i/iter/32768        time:   [20.175 µs 20.519 µs 20.886 µs]
xap_i/xap/32768         time:   [20.981 µs 21.413 µs 21.854 µs]

xap_i/iter/1048576      time:   [2.0699 ms 2.0949 ms 2.1214 ms]
xap_i/xap/1048576       time:   [2.0866 ms 2.1192 ms 2.1541 ms]


COLLECT:
xap_i/iter/1024         time:   [1.1623 µs 1.1722 µs 1.1829 µs]
xap_i/xap/1024          time:   [1.2733 µs 1.2837 µs 1.2941 µs]

xap_i/iter/32768        time:   [71.434 µs 72.159 µs 72.903 µs]
xap_i/xap/32768         time:   [74.546 µs 75.423 µs 76.370 µs]

xap_i/iter/1048576      time:   [2.7622 ms 2.7922 ms 2.8242 ms]
xap_i/xap/1048576       time:   [2.8856 ms 2.9066 ms 2.9278 ms]

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

pub struct Collect;
impl Exp for Collect {
    type Out = Vec<u64>;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        i.collect()
    }
}

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f1(i: u64) -> Option<u64> {
    match i.is_multiple_of(7) {
        true => None,
        false => Some(i + 3),
    }
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter_map(f1);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter_map(f1);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_i");

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
