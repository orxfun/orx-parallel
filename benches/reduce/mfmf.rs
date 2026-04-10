/*

* light & heavy show the intensity of computation
* eN means an input of size 2^N is used

reduce_mfmf/seq/e15_light       time:   [54.684 µs 55.125 µs 55.603 µs]
reduce_mfmf/rayon1/e15_light    time:   [7.0712 ms 7.1686 ms 7.2666 ms]
reduce_mfmf/rayon2/e15_light    time:   [7.9791 ms 8.1189 ms 8.2630 ms]
reduce_mfmf/orx/e15_light       time:   [1.2943 ms 1.3258 ms 1.3617 ms]

reduce_mfmf/seq/e20_light       time:   [2.6323 ms 2.6552 ms 2.6808 ms]
reduce_mfmf/rayon1/e20_light    time:   [10.068 ms 11.214 ms 12.433 ms]
reduce_mfmf/rayon2/e20_light    time:   [16.924 ms 17.365 ms 17.799 ms]
reduce_mfmf/orx/e20_light       time:   [2.6776 ms 2.7611 ms 2.8483 ms]

reduce_mfmf/seq/e15_heavy       time:   [2.3604 ms 2.3842 ms 2.4090 ms]
reduce_mfmf/rayon1/e15_heavy    time:   [9.9925 ms 10.299 ms 10.600 ms]
reduce_mfmf/rayon2/e15_heavy    time:   [10.023 ms 10.515 ms 10.970 ms]
reduce_mfmf/orx/e15_heavy       time:   [2.3577 ms 2.3811 ms 2.4061 ms]

reduce_mfmf/seq/e20_heavy       time:   [86.268 ms 87.874 ms 89.749 ms]
reduce_mfmf/rayon1/e20_heavy    time:   [16.306 ms 16.906 ms 17.513 ms]
reduce_mfmf/rayon2/e20_heavy    time:   [16.252 ms 17.282 ms 18.402 ms]
reduce_mfmf/orx/e20_heavy       time:   [10.698 ms 10.789 ms 10.883 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hint::black_box;

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

fn l_r(a: u64, b: u64) -> u64 {
    a + b
}

fn h_r(a: u64, b: u64) -> u64 {
    let f = black_box(fibonacci(a % FIB_UPPER_BOUND));
    let g = black_box(a + f);
    g + b - f
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

fn f2(a: &u64) -> bool {
    !(2 * a + 11).is_multiple_of(7)
}

fn seq(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input
            .iter()
            .map(m)
            .filter(f)
            .map(h_m2)
            .filter(f2)
            .reduce(h_r),
        false => input
            .iter()
            .map(m)
            .filter(f)
            .map(l_m2)
            .filter(f2)
            .reduce(l_r),
    }
}

fn orx(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input
            .into_par()
            .map(m)
            .filter(f)
            .map(h_m2)
            .filter(f2)
            .reduce(h_r),
        false => input
            .into_par()
            .map(m)
            .filter(f)
            .map(l_m2)
            .filter(f2)
            .reduce(l_r),
    }
}

fn rayon1(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(h_m2)
            .filter(f2)
            .reduce_with(h_r),
        false => input
            .into_par_iter()
            .map(m)
            .filter(f)
            .map(l_m2)
            .filter(f2)
            .reduce_with(l_r),
    }
}

fn rayon2(input: &[u64], h: bool) -> Option<u64> {
    match h {
        true => Some(
            input
                .into_par_iter()
                .map(m)
                .filter(f)
                .map(h_m2)
                .filter(f2)
                .reduce(|| 0, h_r),
        ),
        false => Some(
            input
                .into_par_iter()
                .map(m)
                .filter(f)
                .map(l_m2)
                .filter(f2)
                .reduce(|| 0, l_r),
        ),
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

    let mut group = c.benchmark_group("reduce_mfmf");

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

        group.bench_with_input(BenchmarkId::new("seq", &name), &name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.heavy));
            b.iter(|| seq(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("rayon1", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon1(&input, t.heavy));
            b.iter(|| rayon1(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("rayon2", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon2(&input, t.heavy));
            b.iter(|| rayon2(&input, t.heavy))
        });

        group.bench_with_input(BenchmarkId::new("orx", &name), &name, |b, _| {
            assert_eq!(&expected, &orx(&input, t.heavy));
            b.iter(|| orx(&input, t.heavy))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
