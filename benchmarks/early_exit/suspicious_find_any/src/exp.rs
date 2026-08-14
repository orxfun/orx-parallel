use crate::alg::Method;
use crate::input::{InputVariant, Pos};
use bench_helper::runner::cpu_mix;
use orx_criterion::Experiment;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use std::hint::black_box;

const CPU_MIX_ROUNDS: usize = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub source: u32,
    pub code: u16,
    pub ts: u64,
    pub payload_seed: u64,
    pub signature: u64,
}

pub struct Exp;

fn suspicious_signature(ts: u64, payload_seed: u64) -> u64 {
    cpu_mix(CPU_MIX_ROUNDS, ts ^ payload_seed)
}

fn is_suspicious(event: &Event) -> bool {
    suspicious_signature(event.ts, event.payload_seed) == event.signature
        && black_box(event.code == 911)
}

fn sentinel_index(len: usize, pos: Pos) -> Option<usize> {
    match pos {
        Pos::Early => Some((len / 256).max(1)),
        Pos::Mid => Some((len / 2).saturating_sub(3)),
        Pos::Late => Some(len.saturating_sub(17)),
        Pos::Never => None,
    }
}

fn inputs(len: usize, pos: Pos) -> Vec<Event> {
    const SEED: u64 = 0x8899_AABB_CCDD_EEFF;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);

    let mut events: Vec<_> = (0..len)
        .map(|idx| Event {
            source: idx as u32,
            code: rng.random_range(100..=900),
            ts: 2_000_000_000 + idx as u64,
            payload_seed: rng.random::<u64>() ^ (idx as u64).rotate_right(7),
            signature: rng.random(),
        })
        .collect();

    if let Some(idx) = sentinel_index(len, pos) {
        let base = events[idx];
        events[idx] = Event {
            code: 911,
            signature: suspicious_signature(base.ts, base.payload_seed),
            ..base
        };
    }

    events
}

fn find_seq(input: &[Event]) -> Option<u32> {
    input.iter().find(|e| is_suspicious(e)).map(|e| e.source)
}

fn find_rayon(input: &[Event]) -> Option<u32> {
    input
        .par_iter()
        .find_any(|e| is_suspicious(e))
        .map(|e| e.source)
}

fn find_orx(input: &[Event]) -> Option<u32> {
    input
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .find(|e| is_suspicious(e))
        .map(|e| e.source)
}

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = Vec<Event>;
    type Output = Option<u32>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(input_variant.len(), input_variant.pos)
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match alg_variant {
            Method::Seq => find_seq(input),
            Method::Rayon => find_rayon(input),
            Method::OrxOnce => find_orx(input),
            Method::OrxBasic => find_orx(input),
            Method::OrxRayon => find_orx(input),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(find_seq(input))
    }
}
