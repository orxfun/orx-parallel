/*

* beg & mid & end show where the element to be found is located

first_f/seq/e20_early   time:   [104.15 ns 105.23 ns 106.41 ns]
first_f/rayon/e20_early time:   [2.7674 ms 2.8771 ms 2.9895 ms]
first_f/orx/e20_early   time:   [1.9042 ms 1.9447 ms 1.9878 ms]

first_f/seq/e20_mid     time:   [213.48 µs 218.52 µs 223.92 µs]
first_f/rayon/e20_mid   time:   [20.742 ms 24.835 ms 30.378 ms]
first_f/orx/e20_mid     time:   [2.6632 ms 2.7669 ms 2.8750 ms]

first_f/seq/e20_late    time:   [440.82 µs 447.87 µs 455.68 µs]
first_f/rayon/e20_late  time:   [21.300 ms 22.282 ms 23.321 ms]
first_f/orx/e20_late    time:   [2.1220 ms 2.1562 ms 2.1959 ms]
*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::IntoConcurrentIter;
use orx_parallel::par;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn inputs(len: usize, pos: usize, val: u64) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut vec = Vec::with_capacity(len);
    vec.extend((0..(len - 1)).map(|_| rng.random_range(0..150)));
    vec.insert(pos, val);
    vec
}

fn seq(input: &[u64], value: u64) -> Option<u64> {
    input.iter().filter(|x| **x == value).next().copied()
}

fn orx(input: &[u64], value: u64) -> Option<u64> {
    let iter = input.into_con_iter();
    par(iter).filter(|x| **x == value).first().copied()
}

fn rayon(input: &[u64], value: u64) -> Option<u64> {
    input
        .into_par_iter()
        .filter(|x| **x == value)
        .find_first(|_| true)
        .copied()
}

struct Treat {
    len: usize,
    pos: usize,
    val: u64,
    name: String,
}

fn run(c: &mut Criterion) {
    let treatments = [
        Treat {
            len: 1 << 20,
            pos: 1 << 8,
            name: format!("e{}_early", 20),
            val: 999,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 19) + 7,
            name: format!("e{}_mid", 20),
            val: 999,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 20) - 27,
            name: format!("e{}_late", 20),
            val: 999,
        },
    ];

    let mut group = c.benchmark_group("first_f");

    for t in treatments {
        let input = inputs(t.len, t.pos, t.val);
        let expected = seq(&input, t.val);

        group.bench_with_input(BenchmarkId::new("seq", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.val));
            b.iter(|| seq(&input, t.val))
        });

        group.bench_with_input(BenchmarkId::new("rayon", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &rayon(&input, t.val));
            b.iter(|| rayon(&input, t.val))
        });

        group.bench_with_input(BenchmarkId::new("orx", &t.name), &t.name, |b, _| {
            assert_eq!(&expected, &orx(&input, t.val));
            b.iter(|| orx(&input, t.val))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
