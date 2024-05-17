use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rayon::iter::IntoParallelIterator;

fn predicate(value: usize) -> bool {
    let str = format!("_-{}-_", value);
    let first = str.chars().nth(1) == Some('-');
    let second = str.chars().nth(2) == Some('0');
    let third = str.chars().nth(3) == Some('-');
    first && second && third
}

fn inputs(len: usize, find_idx: usize) -> Vec<usize> {
    let mut vec: Vec<_> = (1..(len + 1)).map(|i| 1000 + 2 * i).collect();
    if let Some(x) = vec.get_mut(find_idx) {
        *x = 0
    }
    vec
}

fn seq(inputs: &[usize]) -> Option<usize> {
    inputs.iter().find(|x| predicate(**x)).copied()
}

fn rayon(inputs: &[usize]) -> Option<usize> {
    use rayon::iter::ParallelIterator;
    inputs
        .into_par_iter()
        .find_first(|x| predicate(**x))
        .copied()
}

fn orx_parallel_default(inputs: &[usize]) -> Option<usize> {
    inputs.into_par().find(|x| predicate(**x)).map(|x| *x.1)
}

fn orx_parallel(inputs: &[usize], num_threads: usize, chunk_size: usize) -> Option<usize> {
    inputs
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .find(|x| predicate(**x))
        .map(|x| *x.1)
}

fn find(c: &mut Criterion) {
    let lengths = [65_536, 262_144];
    let positions = [[34_487, usize::MAX], [147_324, usize::MAX]];
    let params = [(8, 64)];

    let mut group = c.benchmark_group("find");

    for (len, positions) in lengths.iter().zip(positions) {
        for pos in positions {
            let expected = if pos == usize::MAX { None } else { Some(0) };
            let input = inputs(*len, pos);
            let name = format!("n{}-p{}", len, pos);

            group.bench_with_input(BenchmarkId::new("seq", name.clone()), &name, |b, _| {
                b.iter(|| {
                    let result = seq(black_box(&input));
                    assert_eq!(result, expected);
                })
            });

            group.bench_with_input(BenchmarkId::new("rayon", name.clone()), &name, |b, _| {
                b.iter(|| {
                    let result = rayon(black_box(&input));
                    assert_eq!(result, expected);
                })
            });

            group.bench_with_input(
                BenchmarkId::new("orx-parallel-default", name.clone()),
                &name,
                |b, _| {
                    b.iter(|| {
                        let result = orx_parallel_default(black_box(&input));
                        assert_eq!(result, expected);
                    })
                },
            );

            for (t, c) in params {
                let params = format!("orx-parallel-t{}-c{}", t, c);
                group.bench_with_input(BenchmarkId::new(params, name.clone()), &name, |b, _| {
                    b.iter(|| {
                        let result = orx_parallel(black_box(&input), t, c);
                        assert_eq!(result, expected);
                    })
                });
            }
        }
    }

    group.finish();
}

criterion_group!(benches, find);
criterion_main!(benches);
