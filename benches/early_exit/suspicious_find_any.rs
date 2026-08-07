//! Early-exit find-any benchmark for suspicious-event detection pipelines.
//! Simulates mostly normal records with one rare sentinel and measures
//! first-match discovery under arbitrary iteration order.

use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hint::black_box;

const CPU_MIX_ROUNDS: usize = 40;
fn cpu_mix(seed: u64) -> u64 {
    let mut x = black_box(seed ^ 0x9E37_79B9_7F4A_7C15);
    for r in 0..CPU_MIX_ROUNDS {
        let salt = black_box((r as u64 + 1) * 0xA076_1D64_78BD_642F);
        x = black_box(x ^ salt);
        x = black_box(x.rotate_left(9).wrapping_mul(0xD6E8_FD9D_79A1_4E3B));
        x = black_box(x ^ (x >> 27));
    }
    x
}

#[derive(Debug, Clone, Copy)]
enum Pos {
    Early,
    Mid,
    Late,
    Never,
}

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    pos: Pos,
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "pos"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.pos {
                Pos::Early => "early",
                Pos::Mid => "mid",
                Pos::Late => "late",
                Pos::Never => "never",
            }
            .to_string(),
        ]
    }
}

#[derive(Debug)]
enum Method {
    Seq,
    Rayon { nt: usize },
    Orx { nt: usize },
    OrxFixed { nt: usize },
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![match self {
            Self::Seq => "seq".to_string(),
            Self::Rayon { nt } => format!("rayon-{nt}"),
            Self::Orx { nt } => format!("orx-{nt}"),
            Self::OrxFixed { nt } => format!("orx-fixed-{nt}"),
        }]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Event {
    source: u32,
    code: u16,
    ts: u64,
    payload_seed: u64,
    signature: u64,
}

struct Exp;

fn suspicious_signature(ts: u64, payload_seed: u64) -> u64 {
    cpu_mix(ts ^ payload_seed)
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
        let input = input.as_slice();

        match alg_variant {
            Method::Seq => input.iter().find(|e| is_suspicious(e)).map(|e| e.source),
            Method::Rayon { nt } => {
                let pool = ThreadPoolBuilder::new().num_threads(*nt).build().unwrap();
                pool.install(|| {
                    input
                        .into_par_iter()
                        .find_any(|e| is_suspicious(e))
                        .map(|e| e.source)
                })
            }
            Method::Orx { nt } => input
                .into_par()
                .iteration_order(IterationOrder::Arbitrary)
                .num_threads(*nt)
                .find(|e| is_suspicious(e))
                .map(|e| e.source),
            Method::OrxFixed { nt } => input
                .into_par()
                .runner(Runner::fixed(pool::get_global_pool()))
                .iteration_order(IterationOrder::Arbitrary)
                .num_threads(*nt)
                .find(|e| is_suspicious(e))
                .map(|e| e.source),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(input.iter().find(|e| is_suspicious(e)).map(|e| e.source))
    }
}

fn run(c: &mut Criterion) {
    let ns = [14, 18];
    let positions = [Pos::Never, Pos::Late, Pos::Mid, Pos::Early];
    let treatments: Vec<_> = ns
        .into_iter()
        .flat_map(|n| positions.map(|pos| InputVariant { n, pos }))
        .collect();

    let par_variants = |nt: usize| {
        [
            Method::Rayon { nt },
            Method::Orx { nt },
            Method::OrxFixed { nt },
        ]
    };

    let mut variants = vec![Method::Seq];
    variants.extend(par_variants(1));
    variants.extend(par_variants(4));
    variants.extend(par_variants(16));

    Exp.bench(c, "early_exit_suspicious_find_any", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
