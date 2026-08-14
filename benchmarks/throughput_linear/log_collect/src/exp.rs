use crate::{alg::Method, input::InputVariant};
use bench_helper::runner::cpu_mix;
use orx_criterion::Experiment;
use orx_parallel::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

const SEED: u64 = 0x1A2B_3C4D_7788_99AA;

pub struct Exp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogRecord {
    severity: u8,
    code: u16,
    user_id: u32,
    ts: u64,
    payload_len: u16,
    payload_seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projected {
    user_bucket: u16,
    score: u64,
}

fn inputs(len: usize) -> Vec<LogRecord> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);
    (0..len)
        .map(|idx| LogRecord {
            severity: rng.random_range(0..=5),
            code: rng.random_range(100..=899),
            user_id: rng.random_range(1..=200_000),
            ts: 1_700_000_000 + idx as u64,
            payload_len: rng.random_range(32..=1536),
            payload_seed: rng.random::<u64>() ^ (idx as u64).rotate_left(13),
        })
        .collect()
}

fn parse_project_light(record: &LogRecord) -> Projected {
    let parsed = record.payload_seed ^ record.ts.rotate_left(7) ^ record.code as u64;
    Projected {
        user_bucket: (record.user_id % 1024) as u16,
        score: parsed.wrapping_add(record.payload_len as u64),
    }
}

fn parse_project_heavy(record: &LogRecord) -> Projected {
    let rounds = 8 + record.payload_len as usize / 96;
    let parsed = cpu_mix(
        rounds,
        record.payload_seed ^ record.ts ^ ((record.code as u64) << 21),
    );
    Projected {
        user_bucket: (record.user_id % 1024) as u16,
        score: parsed.wrapping_add(record.payload_len as u64),
    }
}

fn keep(record: &LogRecord) -> bool {
    (record.severity >= 3 && !record.code.is_multiple_of(5)) || record.code == 777
}

fn collect_seq(input: &[LogRecord], heavy: bool) -> Vec<Projected> {
    input
        .iter()
        .filter(|record| keep(record))
        .map(|record| {
            if heavy {
                parse_project_heavy(record)
            } else {
                parse_project_light(record)
            }
        })
        .collect()
}

fn collect_rayon(input: &[LogRecord], heavy: bool) -> Vec<Projected> {
    input
        .par_iter()
        .filter(|record| keep(record))
        .map(|record| {
            if heavy {
                parse_project_heavy(record)
            } else {
                parse_project_light(record)
            }
        })
        .collect()
}

fn collect_orx(input: &[LogRecord], heavy: bool) -> Vec<Projected> {
    input
        .par()
        .filter(|record| keep(record))
        .map(|record| {
            if heavy {
                parse_project_heavy(record)
            } else {
                parse_project_light(record)
            }
        })
        .collect()
}

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = Vec<LogRecord>;
    type Output = Vec<Projected>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(input_variant.len())
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match alg_variant {
            Method::Seq => collect_seq(input, input_variant.heavy),
            Method::Rayon => collect_rayon(input, input_variant.heavy),
            Method::OrxOnce | Method::OrxBasic | Method::OrxRayon => {
                collect_orx(input, input_variant.heavy)
            }
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        Some(collect_seq(input, input_variant.heavy))
    }
}
