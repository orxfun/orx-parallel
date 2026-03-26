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
