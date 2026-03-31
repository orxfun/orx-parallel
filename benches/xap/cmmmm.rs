/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_cmmmm/iter/1024     time:   [1.0900 µs 1.0998 µs 1.1101 µs]
xap_cmmmm/xap/1024      time:   [1.2612 µs 1.2733 µs 1.2869 µs]

xap_cmmmm/iter/32768    time:   [34.260 µs 34.711 µs 35.204 µs]
xap_cmmmm/xap/32768     time:   [43.144 µs 43.696 µs 44.280 µs]

xap_cmmmm/iter/1048576  time:   [1.1400 ms 1.1596 ms 1.1787 ms]
xap_cmmmm/xap/1048576   time:   [1.2939 ms 1.3046 ms 1.3165 ms]


COLLECT:
xap_cmmmm/iter/1024     time:   [1.1141 µs 1.1260 µs 1.1386 µs]
xap_cmmmm/xap/1024      time:   [1.1362 µs 1.1486 µs 1.1609 µs]

xap_cmmmm/iter/32768    time:   [33.317 µs 33.575 µs 33.834 µs]
xap_cmmmm/xap/32768     time:   [45.480 µs 46.127 µs 46.875 µs]

xap_cmmmm/iter/1048576  time:   [1.3304 ms 1.3442 ms 1.3596 ms]
xap_cmmmm/xap/1048576   time:   [1.3939 ms 1.4210 ms 1.4464 ms]

TODO: room for performance improvement

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, fun::FnCopied, xap_variants::Id};
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

fn f1(i: u64) -> u64 {
    2 * i + 1
}

fn f2(i: u64) -> u64 {
    (7 * i).saturating_sub(71)
}

fn f3(i: u64) -> i64 {
    i as i64 - 33
}

fn f4(i: i64) -> u64 {
    let x = match i < 0 {
        true => (-i) as u64,
        false => i as u64,
    };
    x * 3 + 5
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().map(f1).map(f2).map(f3).map(f4);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new()
        .mapped(FnCopied::new())
        .map(f1)
        .map(f2)
        .map(f3)
        .map(f4);
    E::out(inputs.iter().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_cmmmm");

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
