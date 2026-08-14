use crate::{alg::Method, input::InputVariant};
use orx_criterion::Experiment;
use orx_parallel::*;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use std::hint::black_box;

const NORMAL_ITERS_LIGHT: u32 = 50;
const NORMAL_ITERS_HEAVY: u32 = 500;
const OUTLIER_MULTIPLIER_MIN: u32 = 10;
const OUTLIER_MULTIPLIER_MAX: u32 = 100;

#[derive(Clone, Copy)]
pub struct WorkItem {
    pub seed: u64,
    pub iterations: u32,
}

pub struct Exp;

fn seed_for_input(variant: &InputVariant) -> u64 {
    let n_flag = if variant.n == (1 << 12) { 0xA1 } else { 0xB2 };
    let heavy_flag = if variant.heavy { 0xC3 } else { 0xD4 };
    let het = variant.heterogeneity_percent as u64;
    0xC0FF_EE12_3456_7890u64 ^ ((n_flag as u64) << 48) ^ ((heavy_flag as u64) << 40) ^ (het << 8)
}

fn build_workload(variant: &InputVariant) -> Vec<WorkItem> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed_for_input(variant));
    let mut items = vec![
        WorkItem {
            seed: 0,
            iterations: match variant.heavy {
                true => NORMAL_ITERS_HEAVY,
                false => NORMAL_ITERS_LIGHT,
            }
        };
        variant.n
    ];

    for (idx, item) in items.iter_mut().enumerate() {
        item.seed = idx as u64 ^ rng.random::<u64>();
    }

    let outlier_count = (variant.n * variant.heterogeneity_percent as usize) / 100;
    if outlier_count == 0 {
        return items;
    }

    let mut indices: Vec<usize> = (0..variant.n).collect();
    indices.shuffle(&mut rng);

    for idx in indices.into_iter().take(outlier_count) {
        let m = rng.random_range(OUTLIER_MULTIPLIER_MIN..=OUTLIER_MULTIPLIER_MAX);
        let iters = items[idx].iterations.saturating_mul(m);
        items[idx].iterations = iters;
    }

    items
}

fn do_work(item: &WorkItem) -> u64 {
    let mut state = item.seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    for _ in 0..item.iterations {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        state ^= state >> 33;
        state = black_box(state);
    }
    state
}

fn max_seq(input: &[WorkItem]) -> Option<u64> {
    input.iter().map(do_work).max()
}

fn max_rayon(input: &[WorkItem]) -> Option<u64> {
    input.par_iter().map(do_work).max()
}

fn max_orx(input: &[WorkItem]) -> Option<u64> {
    input.par().map(do_work).max()
}

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = Vec<WorkItem>;
    type Output = Option<u64>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        build_workload(input_variant)
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match alg_variant {
            Method::Seq => max_seq(input),
            Method::Rayon => max_rayon(input),
            Method::OrxOnce => max_orx(input),
            Method::OrxBasic => max_orx(input),
            Method::OrxRayon => max_orx(input),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(max_seq(input))
    }
}
