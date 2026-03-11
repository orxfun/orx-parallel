use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::xap::{
    Id, Xap, XapCopied, count::iter::FlatMapIterMany, fun::flat_map::FnFlatMap,
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f1_v(i: u64) -> impl IntoIterator<Item = u64> {
    (0..2).map(move |x| x + i + 1).collect::<Vec<_>>()
}

fn f2_v(i: u64) -> impl IntoIterator<Item = u64> {
    (0..8).map(move |x| i * 7 + x).collect::<Vec<_>>()
}

fn main() {
    let n = 10;
    let inputs = inputs(n);

    let mut it = inputs.iter().copied().flat_map(f1_v).flat_map(f2_v);
    // let (a, b) = it.size_hint();
    // let x = it.next();
    // let (a, b) = it.size_hint();
    let v = Vec::from_iter(it);

    let xap = Id::new().copied().flat_map(f1_v).flat_map(f2_v);
    let it = inputs.iter().flat_map(|x| xap.xap(x));
    let v = Vec::from_iter(it);
    println!("{v:?}");
}
