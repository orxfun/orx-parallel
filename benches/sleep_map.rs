use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rayon::iter::IntoParallelIterator;

const SLEEP_PER_MAP_NS: u64 = 1000;

fn fibonacci(n: &u32) -> u32 {
    std::thread::sleep(std::time::Duration::from_nanos(SLEEP_PER_MAP_NS));
    let mut a = 0;
    let mut b = 1;
    for _ in 0..*n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn inputs(len: usize) -> Vec<u32> {
    (0..len).map(|x| x as u32).collect()
}

fn seq(inputs: &[u32]) -> Vec<u32> {
    inputs.iter().map(fibonacci).collect()
}

fn rayon(inputs: &[u32]) -> Vec<u32> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().map(fibonacci).collect()
}

fn orx_parallel(inputs: &[u32], num_threads: usize, chunk_size: usize) -> Vec<u32> {
    inputs
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .map(fibonacci)
        .collect_vec()
}

fn sleep_map(c: &mut Criterion) {
    let treatments = vec![1024 * 4];
    let params = [(8, 64)];

    let mut group = c.benchmark_group("sleep_map");

    for n in &treatments {
        let input = inputs(*n);

        group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
            b.iter(|| seq(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            b.iter(|| rayon(black_box(&input)))
        });

        for (t, c) in params {
            let name = format!("orx-parallel-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(name, n), n, |b, _| {
                b.iter(|| orx_parallel(black_box(&input), t, c))
            });
        }
    }

    group.finish();
}

criterion_group!(benches, sleep_map);
criterion_main!(benches);
