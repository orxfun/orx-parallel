//! Stateful Monte Carlo batch benchmark using per-thread mutable RNG/scratch.
//! Simulates random-walk trajectories from deterministic sample seeds and
//! compares statistics-only vs stats+trace workloads.

use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    with_trace: bool,
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }

    fn steps(&self) -> usize {
        match self.with_trace {
            true => 192,
            false => 96,
        }
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "mode"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.with_trace {
                true => "stats+trace",
                false => "stats",
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
struct Stats {
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

struct Exp;

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 0xD1CE_BA5E_1020_3040;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);
    (0..len)
        .map(|idx| rng.random::<u64>() ^ (idx as u64).rotate_left(17))
        .collect()
}

fn simulate(state: &mut ThreadState, sample_seed: u64, steps: usize, with_trace: bool) -> Stats {
    state.rng = ChaCha8Rng::seed_from_u64(sample_seed);

    let mut pos: i32 = 0;
    let mut min_seen: i32 = 0;
    let mut digest: u64 = 0;

    if with_trace {
        state.trace.clear();
        state.trace.push(0);
    }

    for step_idx in 0..steps {
        let step = match state.rng.random_bool(0.5) {
            true => 1,
            false => -1,
        };
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

fn rayon_stats(input: &[u64], with_trace: bool, steps: usize, num_threads: usize) -> Option<Stats> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        input
            .into_par_iter()
            .map_init(ThreadState::new, |state, seed| {
                simulate(state, *seed, steps, with_trace)
            })
            .reduce_with(merge)
    })
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
            Method::Rayon { nt } => rayon_stats(input, with_trace, steps, *nt),
            Method::Orx { nt } => input
                .as_slice()
                .into_par()
                .num_threads(*nt)
                .use_new(|_| ThreadState::new())
                .map(|state, seed| simulate(state, *seed, steps, with_trace))
                .reduce(|_, a, b| merge(a, b)),
            Method::OrxFixed { nt } => input
                .as_slice()
                .into_par()
                .runner(Runner::fixed(Pool::once(*nt)))
                .num_threads(*nt)
                .use_new(|_| ThreadState::new())
                .map(|state, seed| simulate(state, *seed, steps, with_trace))
                .reduce(|_, a, b| merge(a, b)),
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

    fn validate_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
        output: &Self::Output,
    ) {
        let expected = seq_stats(input, input_variant.with_trace, input_variant.steps());
        assert_eq!(*output, expected);
    }
}

fn run(c: &mut Criterion) {
    let ns = [16, 20];
    let trace_modes = [false, true];
    let treatments: Vec<_> = ns
        .into_iter()
        .flat_map(|n| trace_modes.map(|with_trace| InputVariant { n, with_trace }))
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

    Exp.bench(
        c,
        "stateful_using_monte_carlo_batch",
        &treatments,
        &variants,
    );
}

criterion_group!(benches, run);
criterion_main!(benches);
