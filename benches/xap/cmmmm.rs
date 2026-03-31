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
xap_cmmmm/iter/1024     time:   [1.1191 µs 1.1308 µs 1.1432 µs]
                        change: [+11.658% +13.097% +14.536%] (p = 0.00 < 0.05)
                        Performance has regressed.
xap_cmmmm/xap/1024      time:   [1.1809 µs 1.1943 µs 1.2078 µs]
                        change: [+16.501% +17.924% +19.401%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
xap_cmmmm/iter/32768    time:   [40.355 µs 40.658 µs 40.950 µs]
                        change: [+22.438% +24.327% +26.131%] (p = 0.00 < 0.05)
                        Performance has regressed.
xap_cmmmm/xap/32768     time:   [39.122 µs 39.747 µs 40.496 µs]
                        change: [+24.019% +25.827% +27.684%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking xap_cmmmm/iter/1048576: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.0s, enable flat sampling, or reduce sample count to 50.
xap_cmmmm/iter/1048576  time:   [1.2935 ms 1.3100 ms 1.3287 ms]
                        change: [+20.266% +22.315% +24.433%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking xap_cmmmm/xap/1048576: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.6s, enable flat sampling, or reduce sample count to 60.
xap_cmmmm/xap/1048576   time:   [1.3466 ms 1.3697 ms 1.3930 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::infallible::{Xap, fun::FnCopied, xap_variants::Id};
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
