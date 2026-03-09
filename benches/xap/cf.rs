/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_cf/iter/1024        time:   [594.17 ns 597.77 ns 601.29 ns]
xap_cf/xap/1024         time:   [610.77 ns 614.16 ns 617.46 ns]

xap_cf/iter/32768       time:   [60.599 µs 61.370 µs 62.151 µs]
xap_cf/xap/32768        time:   [57.854 µs 58.594 µs 59.344 µs]

xap_cf/iter/1048576     time:   [3.5042 ms 3.5362 ms 3.5694 ms]
xap_cf/xap/1048576      time:   [3.4321 ms 3.4527 ms 3.4741 ms]


COLLECT:
xap_cf/iter/1024        time:   [1.2009 µs 1.2096 µs 1.2191 µs]
xap_cf/xap/1024         time:   [1.2655 µs 1.2749 µs 1.2841 µs]

xap_cf/iter/32768       time:   [124.33 µs 125.64 µs 127.15 µs]
xap_cf/xap/32768        time:   [137.45 µs 139.17 µs 141.10 µs]

xap_cf/iter/1048576     time:   [5.0014 ms 5.0401 ms 5.0804 ms]
xap_cf/xap/1048576      time:   [4.7660 ms 4.8089 ms 4.8548 ms]

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
