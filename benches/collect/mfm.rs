/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used
* order:
  * _ord means results are collected in order leading to same result as sequential
  * _arb means results are collected in arbitrary order
* container:
  * _vec means, results are collected into a Vec
  * _vv means, results are collected into a Vec<Vec<_>>
  * _ll means, results are collected into a LinkedList<Vec<_>>
  * Note that _ll and _vv 2-dim jagged results in rayon and orx, respectively

col_mfm/seq/e15_light   time:   [84.500 µs 84.833 µs 85.200 µs]
                        change: [−20.112% −19.004% −17.924%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
col_mfm/rayon/e15_light time:   [12.176 ms 12.430 ms 12.697 ms]
                        change: [−41.334% −37.834% −34.293%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
col_mfm/rayon_ll/e15_light
                        time:   [12.292 ms 12.531 ms 12.780 ms]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking col_mfm/orx_ord/e15_light: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.8s, enable flat sampling, or reduce sample count to 50.
col_mfm/orx_ord/e15_light
                        time:   [1.7726 ms 1.7980 ms 1.8263 ms]
                        change: [−34.950% −32.506% −29.874%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
Benchmarking col_mfm/orx_arb/e15_light: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.2s, enable flat sampling, or reduce sample count to 50.
col_mfm/orx_arb/e15_light
                        time:   [1.7049 ms 1.7227 ms 1.7418 ms]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking col_mfm/orx_arb_vv/e15_light: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.3s, enable flat sampling, or reduce sample count to 50.
col_mfm/orx_arb_vv/e15_light
                        time:   [1.7005 ms 1.7282 ms 1.7613 ms]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
col_mfm/seq/e20_light   time:   [3.2454 ms 3.2751 ms 3.3059 ms]
                        change: [−17.742% −16.128% −14.418%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
col_mfm/rayon/e20_light time:   [22.312 ms 22.933 ms 23.565 ms]
                        change: [−37.718% −32.995% −27.674%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
col_mfm/rayon_ll/e20_light
                        time:   [22.037 ms 22.589 ms 23.170 ms]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
col_mfm/orx_ord/e20_light
                        time:   [4.7729 ms 4.8483 ms 4.9215 ms]
                        change: [−51.022% −46.386% −41.564%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  4 (4.00%) low mild
  2 (2.00%) high mild
col_mfm/orx_arb/e20_light
                        time:   [4.8335 ms 4.9155 ms 4.9995 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
col_mfm/orx_arb_vv/e20_light
                        time:   [3.7879 ms 3.8661 ms 3.9436 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
Benchmarking col_mfm/seq/e15_heavy: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.0s, enable flat sampling, or reduce sample count to 50.
col_mfm/seq/e15_heavy   time:   [1.5904 ms 1.6036 ms 1.6179 ms]
                        change: [+13.767% +15.146% +16.579%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
col_mfm/rayon/e15_heavy time:   [13.308 ms 13.549 ms 13.793 ms]
                        change: [−22.885% −19.585% −16.162%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
col_mfm/rayon_ll/e15_heavy
                        time:   [13.909 ms 14.170 ms 14.434 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
col_mfm/orx_ord/e15_heavy
                        time:   [2.5754 ms 2.6283 ms 2.6940 ms]
                        change: [−20.847% −18.289% −15.478%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
col_mfm/orx_arb/e15_heavy
                        time:   [2.4535 ms 2.4807 ms 2.5087 ms]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
col_mfm/orx_arb_vv/e15_heavy
                        time:   [2.4588 ms 2.4817 ms 2.5051 ms]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking col_mfm/seq/e20_heavy: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.9s, or reduce sample count to 80.
col_mfm/seq/e20_heavy   time:   [51.765 ms 52.462 ms 53.173 ms]
                        change: [−0.5034% +3.0732% +6.6704%] (p = 0.09 > 0.05)
                        No change in performance detected.
col_mfm/rayon/e20_heavy time:   [24.027 ms 24.868 ms 25.767 ms]
                        change: [−56.782% −48.317% −38.645%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
col_mfm/rayon_ll/e20_heavy
                        time:   [21.983 ms 22.914 ms 23.895 ms]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
col_mfm/orx_ord/e20_heavy
                        time:   [8.7410 ms 8.8879 ms 9.0384 ms]
                        change: [−47.831% −43.725% −39.485%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
col_mfm/orx_arb/e20_heavy
                        time:   [8.7147 ms 8.8904 ms 9.0696 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
col_mfm/orx_arb_vv/e20_heavy
                        time:   [7.8088 ms 7.9569 ms 8.1102 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::LinkedList;

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

const FIB_UPPER_BOUND: u64 = 301;

fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn m(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

fn f(a: &u64) -> bool {
    !(a + 7).is_multiple_of(11)
}

fn h_m2(x: u64) -> u64 {
    match x {
        999 => 999,
        n => fibonacci(n % FIB_UPPER_BOUND) + 1000,
    }
}

fn l_m2(x: u64) -> u64 {
    match x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

fn seq(input: &[u64], h: bool) -> Vec<u64> {
    match h {
        true => input.iter().map(m).filter(f).map(h_m2).collect(),
        false => input.iter().map(m).filter(f).map(l_m2).collect(),
    }
}

fn orx<C: ParCollectInto<u64>>(input: &[u64], h: bool, order: IterationOrder) -> C {
    match h {
        true => input
            .into_par()
            .iteration_order(order)
            .map(m)
            .filter(f)
            .map(h_m2)
            .collect(),
        false => input
            .into_par()
            .iteration_order(order)
            .map(m)
            .filter(f)
            .map(l_m2)
            .collect(),
    }
}

fn rayon(input: &[u64], h: bool) -> Vec<u64> {
    match h {
        true => input.into_par_iter().map(m).filter(f).map(h_m2).collect(),
        false => input.into_par_iter().map(m).filter(f).map(l_m2).collect(),
    }
}

fn rayon_ll(input: &[u64], h: bool) -> LinkedList<Vec<u64>> {
    match h {
        true => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(h_m2)
            .collect_vec_list(),
        false => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(l_m2)
            .collect_vec_list(),
    }
}

struct Treat {
    len: usize,
    heavy: bool,
}

fn run(c: &mut Criterion) {
    let treatments = [
        Treat {
            len: 1 << 15,
            heavy: false,
        },
        Treat {
            len: 1 << 20,
            heavy: false,
        },
        Treat {
            len: 1 << 15,
            heavy: true,
        },
        Treat {
            len: 1 << 20,
            heavy: true,
        },
    ];

    let mut group = c.benchmark_group("col_mfm");

    for t in treatments {
        let name = format!(
            "e{}_{}",
            t.len.ilog2(),
            match t.heavy {
                true => "heavy",
                false => "light",
            },
        );
        let input = inputs(t.len);
        let expected = seq(&input, t.heavy);
        let mut expected_sorted = expected.clone();
        expected_sorted.sort();

        group.bench_with_input(BenchmarkId::new("seq", &name), &name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.heavy));
            b.iter(|| seq(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("rayon", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon(&input, t.heavy));
            b.iter(|| rayon(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("rayon_ll", &name), &name, |b, _| {
            let mut result: Vec<u64> = rayon_ll(&input, t.heavy)
                .into_iter()
                .flat_map(|x| Vec::from(x).into_iter())
                .collect();
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| rayon_ll(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("orx_ord", &name), &name, |b, _| {
            assert_eq!(
                &expected,
                &orx::<Vec<u64>>(&input, t.heavy, IterationOrder::Ordered)
            );
            b.iter(|| orx::<Vec<u64>>(&input, t.heavy, IterationOrder::Ordered))
        });

        group.bench_with_input(BenchmarkId::new("orx_arb", &name), &name, |b, _| {
            let mut result: Vec<u64> = orx(&input, t.heavy, IterationOrder::Arbitrary);
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| orx::<Vec<u64>>(&input, t.heavy, IterationOrder::Arbitrary))
        });

        group.bench_with_input(BenchmarkId::new("orx_arb_vv", &name), &name, |b, _| {
            let mut result: Vec<u64> =
                orx::<Vec<Vec<_>>>(&input, t.heavy, IterationOrder::Arbitrary)
                    .into_iter()
                    .flatten()
                    .collect();
            result.sort();
            assert_eq!(&expected_sorted, &result);
            b.iter(|| orx::<Vec<Vec<_>>>(&input, t.heavy, IterationOrder::Arbitrary))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
