use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelBridge};

const SEED: u64 = 54487;

fn inputs(len: usize) -> Vec<String> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.gen_range(0..1000))
        .map(|x| x.to_string())
        .collect()
}

fn red(a: String, b: String) -> String {
    format!("{}{}", a, b)
}

fn seq(inputs: &[String]) -> Option<String> {
    inputs.iter().cloned().filter(|x| x.len() > 1).reduce(red)
}

fn rayon_reduce(inputs: &[String]) -> Option<String> {
    use rayon::iter::ParallelIterator;
    Some(
        inputs
            .iter()
            .cloned()
            .filter(|x| x.len() > 1)
            .par_bridge()
            .into_par_iter()
            .reduce(|| String::new(), red),
    )
}

fn rayon_reduce_with(inputs: &[String]) -> Option<String> {
    use rayon::iter::ParallelIterator;
    inputs
        .iter()
        .cloned()
        .filter(|x| x.len() > 1)
        .par_bridge()
        .into_par_iter()
        .reduce_with(red)
}

fn orx_parallel_reduce(inputs: &[String], num_threads: usize, chunk_size: usize) -> Option<String> {
    inputs
        .iter()
        .cloned()
        .filter(|x| x.len() > 1)
        .par()
        .num_threads(num_threads)
        .chunk_size(chunk_size)
        .reduce(red)
}

fn orx_parallel_fold(inputs: &[String], num_threads: usize, chunk_size: usize) -> Option<String> {
    Some(
        inputs
            .iter()
            .cloned()
            .filter(|x| x.len() > 1)
            .par()
            .num_threads(num_threads)
            .chunk_size(chunk_size)
            .fold(|| String::new(), red),
    )
}

fn orx_parallel_default(inputs: &[String]) -> Option<String> {
    inputs
        .iter()
        .cloned()
        .filter(|x| x.len() > 1)
        .par()
        .reduce(red)
}

fn into_sorted_chars(result: Option<String>) -> Vec<char> {
    let mut vec: Vec<_> = result.unwrap().chars().collect();
    vec.sort();
    vec
}

fn iter_reduce_concat(c: &mut Criterion) {
    let lengths = [1024, 1024 * 4];

    let mut group = c.benchmark_group("iter_reduce_concat");

    for len in lengths {
        let input = inputs(len);
        let name = format!("n{}", len);
        let expected = into_sorted_chars(seq(&input));

        group.bench_with_input(BenchmarkId::new("seq", name.clone()), &name, |b, _| {
            assert_eq!(expected, into_sorted_chars(seq(&input)));
            b.iter(|| {
                let _ = seq(&input);
            })
        });

        group.bench_with_input(
            BenchmarkId::new("rayon_reduce", name.clone()),
            &name,
            |b, _| {
                assert_eq!(expected, into_sorted_chars(rayon_reduce(&input)));
                b.iter(|| {
                    let _ = rayon_reduce(&input);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rayon_reduce_with", name.clone()),
            &name,
            |b, _| {
                assert_eq!(expected, into_sorted_chars(rayon_reduce_with(&input)));
                b.iter(|| {
                    let _ = rayon_reduce_with(&input);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("orx-parallel-default", name.clone()),
            &name,
            |b, _| {
                assert_eq!(expected, into_sorted_chars(orx_parallel_default(&input)));
                b.iter(|| {
                    let _ = orx_parallel_default(&input);
                })
            },
        );

        let t = 8;
        let c = 1024;
        let par_str = format!("orx-parallel-reduce-t{}-c{}", t, c);
        group.bench_with_input(BenchmarkId::new(par_str, name.clone()), &name, |b, _| {
            assert_eq!(
                expected,
                into_sorted_chars(orx_parallel_reduce(&input, t, c))
            );
            b.iter(|| {
                let _ = orx_parallel_reduce(black_box(&input), t, c);
            })
        });

        let par_str = format!("orx-parallel-fold-t{}-c{}", t, c);
        group.bench_with_input(BenchmarkId::new(par_str, name.clone()), &name, |b, _| {
            assert_eq!(expected, into_sorted_chars(orx_parallel_fold(&input, t, c)));
            b.iter(|| {
                let _ = orx_parallel_fold(black_box(&input), t, c);
            })
        });
    }

    group.finish();
}

criterion_group!(benches, iter_reduce_concat);
criterion_main!(benches);
