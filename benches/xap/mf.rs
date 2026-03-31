/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_mf/iter/1024        time:   [703.52 ns 714.69 ns 725.10 ns]
xap_mf/xap/1024         time:   [696.58 ns 701.65 ns 707.38 ns]

xap_mf/iter/32768       time:   [60.355 µs 61.251 µs 62.254 µs]
xap_mf/xap/32768        time:   [66.071 µs 67.063 µs 68.029 µs]

xap_mf/iter/1048576     time:   [3.6984 ms 3.7298 ms 3.7628 ms]
xap_mf/xap/1048576      time:   [3.5531 ms 3.5740 ms 3.5954 ms]


COLLECT:
xap_mf/iter/1024        time:   [1.2490 µs 1.2644 µs 1.2810 µs]
xap_mf/xap/1024         time:   [1.7197 µs 1.7335 µs 1.7479 µs]

xap_mf/iter/32768       time:   [127.19 µs 127.99 µs 128.84 µs]
xap_mf/xap/32768        time:   [119.33 µs 120.10 µs 120.87 µs]

xap_mf/iter/1048576     time:   [5.1593 ms 5.2029 ms 5.2485 ms]
xap_mf/xap/1048576      time:   [4.4280 ms 4.4606 ms 4.4947 ms

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

fn f2(i: &u64) -> bool {
    !i.is_multiple_of(3)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().map(f1).filter(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().mapped(FnCopied::new()).map(f1).filter(f2);
    E::out(inputs.iter().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_mf");

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
