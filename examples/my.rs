use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{
    Id, Xap, XapCopied, count::iter::FlatMapIterMany, fun::flat_map::FnFlatMap,
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

fn inputs(len: usize) -> Vec<u64> {
    return (0..len as u64).collect();
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f1(i: u64) -> impl IntoIterator<Item = u64> {
    (2u64..5)
        // .map(move |x| i + 2 * x as u64 + 5)
        .map(move |x| i + x - 2)
        .filter(|x| !x.is_multiple_of(3))
}

fn f2(i: u64) -> impl IntoIterator<Item = u64> {
    (6u64..7)
        .map(move |x| 5 * (i + x - 6))
        .filter(|x| !x.is_multiple_of(3))
}

fn main() {
    let n = 2;
    let inputs = inputs(n);

    // let iter = inputs.iter().copied().flat_map(f1).flat_map(f2);
    // let v = Iterator::sum::<u64>(iter);

    let iter = FlatMapIterMany::new(inputs.iter().copied(), FnFlatMap::new(f1));
    let iter = FlatMapIterMany::new(iter, FnFlatMap::new(f2));
    let v: u64 = iter.sum();

    println!("{v:?}");
}
