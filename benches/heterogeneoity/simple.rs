use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn run(c: &mut Criterion) {
    // let treatments = [Input { n: 10 }, Input { n: 15 }, Input { n: 20 }];

    // let variants: Vec<_> = all::<Method>().collect();

    // Exp.bench(c, "first_id", &treatments, &variants);
}
criterion_group!(benches, run);
criterion_main!(benches);
