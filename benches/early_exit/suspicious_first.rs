//! Early-exit first-match benchmark for suspicious-event detection pipelines.
//! Simulates mostly normal records with one rare sentinel and measures ordered
//! first-hit latency across sequential and parallel execution strategies.

use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hint::black_box;

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
    num_threads: usize,
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "pos", "nt"]
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
            self.num_threads.to_string(),
        ]
    }
}

#[derive(Debug, Sequence)]
enum Method {
    Seq,
    Rayon,
    Orx,
    OrxFixed,
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            match self {
                Self::Seq => "seq",
                Self::Rayon => "rayon",
                Self::Orx => "orx",
                Self::OrxFixed => "orx-fixed",
            }
            .to_string(),
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Event {
    source: u32,
    code: u16,
    ts: u64,
    payload_seed: u64,
    signature: u8,
}

struct Exp;

fn cpu_mix(seed: u64, rounds: usize) -> u64 {
    let mut x = black_box(seed ^ 0x9E37_79B9_7F4A_7C15);
    for r in 0..rounds {
        let salt = black_box((r as u64 + 1) * 0xA076_1D64_78BD_642F);
        x ^= salt;
        x = x.rotate_left(9).wrapping_mul(0xD6E8_FD9D_79A1_4E3B);
        x ^= x >> 27;
    }
    x
}

fn suspicious_signature(ts: u64, payload_seed: u64) -> u8 {
    (cpu_mix(ts ^ payload_seed, 7) & 0xFF) as u8
}

fn is_suspicious(event: &Event) -> bool {
    event.code == 911 && suspicious_signature(event.ts, event.payload_seed) == event.signature
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
    const SEED: u64 = 0x1122_3344_5566_7788;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);

    let mut events: Vec<_> = (0..len)
        .map(|idx| Event {
            source: idx as u32,
            code: rng.random_range(100..=900),
            ts: 1_900_000_000 + idx as u64,
            payload_seed: rng.random::<u64>() ^ (idx as u64).rotate_left(11),
            signature: rng.random::<u8>(),
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
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let input = input.as_slice();

        match alg_variant {
            Method::Seq => input.iter().find(|e| is_suspicious(e)).map(|e| e.source),
            Method::Rayon => {
                let pool = ThreadPoolBuilder::new()
                    .num_threads(input_variant.num_threads)
                    .build()
                    .unwrap();
                pool.install(|| {
                    input
                        .into_par_iter()
                        .find_first(|e| is_suspicious(e))
                        .map(|e| e.source)
                })
            }
            Method::Orx => input
                .into_par()
                .num_threads(input_variant.num_threads)
                .filter(|e| is_suspicious(e))
                .first()
                .map(|e| e.source),
            Method::OrxFixed => input
                .into_par()
                .runner(Runner::fixed(Pool::default(input_variant.num_threads)))
                .num_threads(input_variant.num_threads)
                .filter(|e| is_suspicious(e))
                .first()
                .map(|e| e.source),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(input.iter().find(|e| is_suspicious(e)).map(|e| e.source))
    }
}

fn run(c: &mut Criterion) {
    let num_threads_options = [4, 16];
    let treatments: Vec<_> = num_threads_options
        .iter()
        .flat_map(|&num_threads| {
            [
                InputVariant {
                    n: 16,
                    pos: Pos::Early,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    pos: Pos::Early,
                    num_threads,
                },
                InputVariant {
                    n: 16,
                    pos: Pos::Mid,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    pos: Pos::Mid,
                    num_threads,
                },
                InputVariant {
                    n: 16,
                    pos: Pos::Late,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    pos: Pos::Late,
                    num_threads,
                },
                InputVariant {
                    n: 16,
                    pos: Pos::Never,
                    num_threads,
                },
                InputVariant {
                    n: 20,
                    pos: Pos::Never,
                    num_threads,
                },
            ]
        })
        .collect();

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "early_exit_suspicious_first", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
