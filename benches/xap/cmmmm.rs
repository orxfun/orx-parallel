/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_cmmmm/iter/1024     time:   [969.98 ns 980.59 ns 991.56 ns]
xap_cmmmm/xap/1024      time:   [964.62 ns 971.19 ns 978.13 ns]

xap_cmmmm/iter/32768    time:   [30.970 µs 31.162 µs 31.371 µs]
xap_cmmmm/xap/32768     time:   [30.383 µs 30.544 µs 30.718 µs]

xap_cmmmm/iter/1048576  time:   [1.0237 ms 1.0314 ms 1.0394 ms]
xap_cmmmm/xap/1048576   time:   [1.0076 ms 1.0147 ms 1.0229 ms]


COLLECT:
xap_cmmmm/iter/1024     time:   [1.0112 µs 1.0229 µs 1.0346 µs]
xap_cmmmm/xap/1024      time:   [1.0183 µs 1.0267 µs 1.0357 µs]

xap_cmmmm/iter/32768    time:   [31.933 µs 32.417 µs 32.890 µs]
xap_cmmmm/xap/32768     time:   [31.985 µs 32.238 µs 32.488 µs]

xap_cmmmm/iter/1048576  time:   [1.0766 ms 1.0845 ms 1.0930 ms]
xap_cmmmm/xap/1048576   time:   [1.0820 ms 1.0906 ms 1.1000 ms]

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
    let xap = Id::new().copied().map(f1).map(f2).map(f3).map(f4);
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
