/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_fff/iter/1024       time:   [990.29 ns 999.79 ns 1.0093 µs]
xap_fff/xap/1024        time:   [1.0829 µs 1.0910 µs 1.0992 µs]

xap_fff/iter/32768      time:   [110.33 µs 111.40 µs 112.48 µs]
xap_fff/xap/32768       time:   [124.87 µs 125.85 µs 126.86 µs]

xap_fff/iter/1048576    time:   [4.5769 ms 4.6066 ms 4.6372 ms]
xap_fff/xap/1048576     time:   [4.6233 ms 4.6516 ms 4.6814 ms]


COLLECT:
xap_fff/iter/1024       time:   [1.8569 µs 1.8686 µs 1.8804 µs]
xap_fff/xap/1024        time:   [1.9658 µs 1.9815 µs 1.9972 µs]

xap_fff/iter/32768      time:   [170.36 µs 172.01 µs 173.87 µs]
xap_fff/xap/32768       time:   [173.34 µs 175.35 µs 177.82 µs]

xap_fff/iter/1048576    time:   [6.3637 ms 6.4159 ms 6.4690 ms]
xap_fff/xap/1048576     time:   [6.4110 ms 6.4594 ms 6.5084 ms]

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

fn f2(i: &u64) -> bool {
    !(i + 7).is_multiple_of(11)
}

fn f3(i: &u64) -> bool {
    !(i + 11).is_multiple_of(5)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().filter(f1).filter(f2).filter(f3);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().filter(f1).filter(f2).filter(f3);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_fff");

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
