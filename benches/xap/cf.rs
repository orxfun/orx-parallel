/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_cf/iter/1024        time:   [643.08 ns 648.03 ns 653.31 ns]
xap_cf/xap/1024         time:   [693.92 ns 704.40 ns 713.37 ns]

xap_cf/iter/32768       time:   [62.618 µs 63.445 µs 64.362 µs]
xap_cf/xap/32768        time:   [61.319 µs 62.349 µs 63.388 µs]

xap_cf/iter/1048576     time:   [3.7464 ms 3.7913 ms 3.8386 ms]
xap_cf/xap/1048576      time:   [3.9141 ms 3.9763 ms 4.0417 ms]


COLLECT:
xap_cf/iter/1024        time:   [1.2042 µs 1.2173 µs 1.2312 µs]
xap_cf/xap/1024         time:   [1.2544 µs 1.2668 µs 1.2799 µs]

xap_cf/iter/32768       time:   [131.29 µs 132.84 µs 134.54 µs]
xap_cf/xap/32768        time:   [136.75 µs 138.26 µs 139.92 µs]

xap_cf/iter/1048576     time:   [5.0271 ms 5.0646 ms 5.1032 ms]
xap_cf/xap/1048576      time:   [4.9079 ms 4.9462 ms 4.9860 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap, XapCopied};
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

fn f1(i: &u64) -> bool {
    !i.is_multiple_of(3)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter(f1);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().copied().filter(f1);
    E::out(inputs.iter().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_cf");

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
