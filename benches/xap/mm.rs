/*

The goal of this benchmark is to measure the overhead of Xap abstraction.
Operations after iteration are kept to be as simple as possible to observe the overhead.

SUM:
xap_mm/iter/1024        time:   [580.11 ns 585.77 ns 591.85 ns]
                        change: [−9.0074% −7.8452% −6.7494%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
xap_mm/xap/1024         time:   [581.31 ns 586.08 ns 591.00 ns]
                        change: [−2.6432% −1.5160% −0.2956%] (p = 0.01 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
xap_mm/iter/32768       time:   [19.707 µs 19.969 µs 20.227 µs]
                        change: [+0.9473% +2.2441% +3.4620%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
xap_mm/xap/32768        time:   [19.462 µs 19.619 µs 19.783 µs]
                        change: [+0.3403% +1.4197% +2.5523%] (p = 0.01 < 0.05)
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
xap_mm/iter/1048576     time:   [662.69 µs 669.02 µs 676.03 µs]
                        change: [−0.0647% +1.3344% +2.7895%] (p = 0.06 > 0.05)
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
xap_mm/xap/1048576      time:   [657.29 µs 663.99 µs 671.09 µs]


COLLECT:
xap_mm/iter/1024        time:   [578.81 ns 583.07 ns 587.22 ns]
xap_mm/xap/1024         time:   [591.98 ns 595.90 ns 599.96 ns]

xap_mm/iter/32768       time:   [17.677 µs 17.821 µs 17.969 µs]
xap_mm/xap/32768        time:   [17.633 µs 17.822 µs 18.025 µs]

xap_mm/iter/1048576     time:   [651.95 µs 659.59 µs 666.98 µs]
xap_mm/xap/1048576      time:   [662.66 µs 671.09 µs 679.71 µs]

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

fn f1(i: u64) -> u64 {
    2 * i + 1
}

fn f2(i: u64) -> u64 {
    (7 * i).saturating_sub(71)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().map(f1).map(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().map(f1).map(f2);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15, 1 << 20];

    let mut group = c.benchmark_group("xap_mm");

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
