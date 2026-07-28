use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::{Sequence, all};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::hint::black_box;

const NUM_THREADS: usize = 16;
const HOMOGENEOUS_WORK: usize = 96;
const HETEROGENEOUS_LIGHT_WORK: usize = 24;
const HETEROGENEOUS_MEDIUM_WORK: usize = 192;
const HETEROGENEOUS_HEAVY_WORK: usize = 1536;

#[derive(Clone, Copy, Debug)]
enum TaskKind {
    Homogeneous,
    Heterogeneous,
}

impl TaskKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Homogeneous => "homogeneous",
            Self::Heterogeneous => "heterogeneous",
        }
    }
}

#[derive(Clone, Copy)]
struct Input {
    len_exp: usize,
    task_kind: TaskKind,
}

impl Factors for Input {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "tasks"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.len_exp),
            self.task_kind.as_str().to_string(),
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

struct BenchInput {
    data: Vec<u64>,
    pool: rayon_core::ThreadPool,
}

struct Exp;

fn values(len: usize) -> Vec<u64> {
    const SEED: u64 = 0xD1CE_BA5E;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);
    (0..len)
        .map(|idx| rng.random::<u64>() ^ ((idx as u64 + 1) * 0x9E37_79B9_7F4A_7C15))
        .collect()
}

fn work_units(value: u64, task_kind: TaskKind) -> usize {
    match task_kind {
        TaskKind::Homogeneous => HOMOGENEOUS_WORK,
        TaskKind::Heterogeneous => match value & 0x0f {
            0 => HETEROGENEOUS_HEAVY_WORK,
            1..=3 => HETEROGENEOUS_MEDIUM_WORK,
            _ => HETEROGENEOUS_LIGHT_WORK,
        },
    }
}

fn expensive_map(value: &u64, task_kind: TaskKind) -> u64 {
    let mut acc = black_box(*value ^ 0xA076_1D64_78BD_642F);
    let rounds = work_units(*value, task_kind);

    for round in 0..rounds {
        let salt = black_box((round as u64 + 1) * 0xE703_7ED1_A0B4_28DB);
        acc = acc.rotate_left(11) ^ salt;
        acc = acc.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        acc ^= acc >> 29;
    }

    acc ^ (*value).rotate_left(7)
}

fn selective_filter(value: &u64) -> bool {
    let folded = value ^ value.rotate_right(17);
    !folded.count_ones().is_multiple_of(3)
}

fn reduce_sum(a: u64, b: u64) -> u64 {
    a.wrapping_add(b)
}

impl Experiment for Exp {
    type InputFactors = Input;

    type AlgFactors = Method;

    type Input = BenchInput;

    type Output = Option<u64>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        let len = 1usize << input_variant.len_exp;
        let data = values(len);
        let pool = Pool::rayon(NUM_THREADS).expect("failed to build rayon thread pool");
        BenchInput { data, pool }
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let task_kind = input_variant.task_kind;
        let data = input.data.as_slice();

        match alg_variant {
            Method::Seq => data
                .iter()
                .map(|value| expensive_map(value, task_kind))
                .filter(selective_filter)
                .reduce(reduce_sum),
            Method::Rayon => input.pool.install(|| {
                data.par_iter()
                    .map(|value| expensive_map(value, task_kind))
                    .filter(selective_filter)
                    .reduce_with(reduce_sum)
            }),
            Method::Orx => data
                .into_par()
                .runner(Runner::adaptive(&input.pool))
                .num_threads(NUM_THREADS)
                .map(|value| expensive_map(value, task_kind))
                .filter(selective_filter)
                .reduce(reduce_sum),
            Method::OrxFixed => data
                .into_par()
                .runner(Runner::fixed(&input.pool))
                .num_threads(NUM_THREADS)
                .map(|value| expensive_map(value, task_kind))
                .filter(selective_filter)
                .reduce(reduce_sum),
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        let task_kind = input_variant.task_kind;
        Some(
            input
                .data
                .iter()
                .map(|value| expensive_map(value, task_kind))
                .filter(selective_filter)
                .reduce(reduce_sum),
        )
    }
}

fn run(c: &mut Criterion) {
    let task_kind = [TaskKind::Homogeneous, TaskKind::Heterogeneous];
    let len_exp = [16, 20];

    let treatments: Vec<_> = task_kind
        .into_iter()
        .flat_map(|task_kind| {
            len_exp
                .into_iter()
                .map(move |len_exp| Input { len_exp, task_kind })
        })
        .collect();

    let variants: Vec<_> = all::<Method>().collect();

    Exp.bench(c, "runner_mf_r", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
