/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_ii/iter/1024        time:   [676.74 ns 680.75 ns 684.80 ns]
xap_ii/xap/1024         time:   [718.92 ns 723.10 ns 727.17 ns]

xap_ii/iter/32768       time:   [46.433 µs 46.824 µs 47.183 µs]
xap_ii/xap/32768        time:   [50.096 µs 50.623 µs 51.131 µs]

xap_ii/iter/1048576     time:   [1.9990 ms 2.0115 ms 2.0244 ms]
xap_ii/xap/1048576      time:   [2.1596 ms 2.1772 ms 2.1951 ms]


COLLECT:
xap_ii/iter/1024        time:   [1.1962 µs 1.2077 µs 1.2186 µs]
xap_ii/xap/1024         time:   [1.6818 µs 1.6942 µs 1.7082 µs]

xap_ii/iter/32768       time:   [137.64 µs 138.67 µs 139.72 µs]
xap_ii/xap/32768        time:   [153.08 µs 154.52 µs 156.11 µs]

xap_ii/iter/1048576     time:   [5.4428 ms 5.4880 ms 5.5343 ms]
xap_ii/xap/1048576      time:   [5.3084 ms 5.3591 ms 5.4121 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{Id, Xap};
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

fn f1(i: u64) -> Option<u64> {
    match i.is_multiple_of(7) {
        true => None,
        false => Some(i + 3),
    }
}

fn f2(i: u64) -> Option<u64> {
    match i.is_multiple_of(3) {
        true => None,
        false => Some(2 * i),
    }
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter_map(f1).filter_map(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter_map(f1).filter_map(f2);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_ii");

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
