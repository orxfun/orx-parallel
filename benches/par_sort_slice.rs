use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::{sort::slice::SortChunks, *};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::slice::ParallelSliceMut;
use std::{hint::black_box, num::NonZeroUsize};

type Val = String;

fn create_input_and_sorted<T: Clone + Ord>(
    len: usize,
    value_at: impl Fn(usize) -> T,
    number_of_swaps: usize,
) -> (Vec<T>, Vec<T>) {
    let mut input: Vec<_> = (0..len).map(value_at).collect();
    let mut sorted = input.clone();
    sorted.sort();
    if len > 0 {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..number_of_swaps {
            let i = rng.random_range(0..len);
            let j = rng.random_range(0..len);
            input.swap(i, j);
        }
    }
    (input, sorted)
}

fn seq(inputs: &mut [Val]) {
    inputs.sort();
}

fn rayon(inputs: &mut [Val]) {
    inputs.par_sort();
}

fn orx_with_queue(inputs: &mut [Val], nt: usize) {
    orx_parallel::sort::slice::sort(
        &mut StdDefaultPool::default(),
        NonZeroUsize::new(nt).unwrap(),
        inputs,
        SortChunks::SeqWithPriorityQueue,
    );
}

fn orx_with_queue_ptrs(inputs: &mut [Val], nt: usize) {
    orx_parallel::sort::slice::sort(
        &mut StdDefaultPool::default(),
        NonZeroUsize::new(nt).unwrap(),
        inputs,
        SortChunks::SeqWithPriorityQueuePtrs,
    );
}

fn orx_with_vec(inputs: &mut [Val], nt: usize) {
    orx_parallel::sort::slice::sort(
        &mut StdDefaultPool::default(),
        NonZeroUsize::new(nt).unwrap(),
        inputs,
        SortChunks::SeqWithVec,
    );
}

fn orx_with_merge(inputs: &mut [Val], depth: usize) {
    orx_parallel::sort::slice::sort2(inputs, depth);
}

fn run(c: &mut Criterion) {
    let lengths = [65_536 * 1, 65_536 * 2, 65_536 * 4, 65_536 * 8];
    let lengths = [65_536 * 8, 65_536 * 16, 65_536 * 32];
    let num_swaps = [65_536 * 64];

    let mut group = c.benchmark_group("par_sort_slice");

    let value_at = |i: usize| i.to_string();

    for len in &lengths {
        for swaps in &num_swaps {
            let (input, sorted) = create_input_and_sorted(*len, value_at, *swaps);

            group.bench_with_input(BenchmarkId::new("seq", len), len, |b, _| {
                let mut input = input.clone();
                b.iter(|| seq(black_box(&mut input)));
                assert_eq!(&input, &sorted);
            });

            group.bench_with_input(BenchmarkId::new("rayon", len), len, |b, _| {
                let mut input = input.clone();
                b.iter(|| rayon(black_box(&mut input)));
                assert_eq!(&input, &sorted);
            });

            group.bench_with_input(BenchmarkId::new("orx_with_queue", len), len, |b, _| {
                let mut input = input.clone();
                b.iter(|| orx_with_queue(black_box(&mut input), 32));
                assert_eq!(&input, &sorted);
            });

            group.bench_with_input(BenchmarkId::new("orx_with_merge2", len), len, |b, _| {
                let mut input = input.clone();
                b.iter(|| orx_with_merge(black_box(&mut input), 2));
                assert_eq!(&input, &sorted);
            });

            group.bench_with_input(BenchmarkId::new("orx_with_merge4", len), len, |b, _| {
                let mut input = input.clone();
                b.iter(|| orx_with_merge(black_box(&mut input), 4));
                assert_eq!(&input, &sorted);
            });

            // group.bench_with_input(BenchmarkId::new("orx_with_merge6", len), len, |b, _| {
            //     let mut input = input.clone();
            //     b.iter(|| orx_with_merge(black_box(&mut input), 6));
            //     assert_eq!(&input, &sorted);
            // });

            group.bench_with_input(BenchmarkId::new("orx_with_merge8", len), len, |b, _| {
                let mut input = input.clone();
                b.iter(|| orx_with_merge(black_box(&mut input), 8));
                assert_eq!(&input, &sorted);
            });

            group.bench_with_input(BenchmarkId::new("orx_with_merge16", len), len, |b, _| {
                let mut input = input.clone();
                b.iter(|| orx_with_merge(black_box(&mut input), 16));
                assert_eq!(&input, &sorted);
            });

            // group.bench_with_input(BenchmarkId::new("orx_with_merge32", len), len, |b, _| {
            //     let mut input = input.clone();
            //     b.iter(|| orx_with_merge(black_box(&mut input), 32));
            //     assert_eq!(&input, &sorted);
            // });
        }
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
