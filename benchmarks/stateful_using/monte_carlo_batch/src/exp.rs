use crate::{alg::Method, input::InputVariant};
use orx_criterion::Experiment;
use orx_parallel::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

pub struct Exp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    sum_final: i64,
    min_seen: i32,
    trace_digest: u64,
    samples: u64,
}

fn merge(a: Stats, b: Stats) -> Stats {
    Stats {
        sum_final: a.sum_final + b.sum_final,
        min_seen: a.min_seen.min(b.min_seen),
        trace_digest: a.trace_digest ^ b.trace_digest,
        samples: a.samples + b.samples,
    }
}

struct ThreadState {
    rng: ChaCha8Rng,
    trace: Vec<i32>,
}

impl ThreadState {
    fn new() -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(0),
            trace: Vec::new(),
        }
    }
}

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 0xD1CE_BA5E_1020_3040;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);
    (0..len)
        .map(|idx| rng.random::<u64>() ^ (idx as u64).rotate_left(17))
        .collect()
}

fn simulate(state: &mut ThreadState, sample_seed: u64, steps: usize, with_trace: bool) -> Stats {
    state.rng = ChaCha8Rng::seed_from_u64(sample_seed);
    let mut pos = 0_i32;
    let mut min_seen = 0_i32;
    let mut digest = 0_u64;

    if with_trace {
        state.trace.clear();
        state.trace.push(0);
    }

    for step_idx in 0..steps {
        let step = if state.rng.random_bool(0.5) { 1 } else { -1 };
        pos += step;
        min_seen = min_seen.min(pos);
        if with_trace {
            state.trace.push(pos);
            digest = digest
                .wrapping_mul(0x9E37_79B9)
                .wrapping_add((pos as i64 as u64) ^ step_idx as u64);
        }
    }

    Stats {
        sum_final: pos as i64,
        min_seen,
        trace_digest: digest,
        samples: 1,
    }
}

fn seq_stats(input: &[u64], with_trace: bool, steps: usize) -> Option<Stats> {
    let mut state = ThreadState::new();
    input
        .iter()
        .map(|seed| simulate(&mut state, *seed, steps, with_trace))
        .reduce(merge)
}

fn rayon_stats(input: &[u64], with_trace: bool, steps: usize) -> Option<Stats> {
    input
        .par_iter()
        .map_init(ThreadState::new, |state, seed| {
            simulate(state, *seed, steps, with_trace)
        })
        .reduce_with(merge)
}

fn orx_stats(input: &[u64], with_trace: bool, steps: usize) -> Option<Stats> {
    input
        .par()
        .use_new(|_| ThreadState::new())
        .map(|state, seed| simulate(state, *seed, steps, with_trace))
        .reduce(|_, a, b| merge(a, b))
}

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = Vec<u64>;
    type Output = Option<Stats>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(input_variant.len())
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let with_trace = input_variant.with_trace;
        let steps = input_variant.steps();
        match alg_variant {
            Method::Seq => seq_stats(input, with_trace, steps),
            Method::Rayon => rayon_stats(input, with_trace, steps),
            Method::OrxOnce | Method::OrxBasic | Method::OrxRayon => {
                orx_stats(input, with_trace, steps)
            }
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        Some(seq_stats(
            input,
            input_variant.with_trace,
            input_variant.steps(),
        ))
    }
}
