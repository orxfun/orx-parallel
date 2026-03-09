/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_f/iter/1024         time:   [522.96 ns 526.22 ns 529.81 ns]
xap_f/xap/1024          time:   [529.05 ns 531.50 ns 534.01 ns]

xap_f/iter/32768        time:   [79.767 µs 80.427 µs 81.127 µs]
xap_f/xap/32768         time:   [79.418 µs 79.976 µs 80.593 µs]

xap_f/iter/1048576      time:   [3.2563 ms 3.2871 ms 3.3233 ms]
xap_f/xap/1048576       time:   [3.3664 ms 3.3969 ms 3.4298 ms]


COLLECT:
xap_f/iter/1024         time:   [1.0567 µs 1.0647 µs 1.0725 µs]
xap_f/xap/1024          time:   [1.4865 µs 1.4936 µs 1.5017 µs]

xap_f/iter/32768        time:   [110.64 µs 111.35 µs 112.09 µs]
xap_f/xap/32768         time:   [102.99 µs 103.64 µs 104.34 µs]

xap_f/iter/1048576      time:   [4.4457 ms 4.4845 ms 4.5262 ms]
xap_f/xap/1048576       time:   [3.7234 ms 3.7447 ms 3.7660 ms]

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

fn f1(i: &u64) -> bool {
    !i.is_multiple_of(3)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter(f1);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter(f1);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_f");

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
