use crate::{alg::Method, input::InputVariant};
use bench_helper::runner::cpu_mix;
use orx_criterion::Experiment;
use orx_parallel::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

const SEED: u64 = 0x55AA_A1A1_9090_F0F0;

pub struct Exp;

#[derive(Clone, Copy, Debug)]
pub struct LogRecord { severity: u8, code: u16, user_id: u32, ts: u64, payload_len: u16, payload_seed: u64 }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Agg { err: u64, warn: u64, info: u64, checksum: u64 }

fn inputs(len: usize) -> Vec<LogRecord> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);
    (0..len).map(|idx| LogRecord {
        severity: rng.random_range(0..=5), code: rng.random_range(100..=899), user_id: rng.random_range(1..=200_000),
        ts: 1_700_000_000 + idx as u64, payload_len: rng.random_range(32..=1536),
        payload_seed: rng.random::<u64>() ^ (idx as u64).rotate_right(9),
    }).collect()
}

fn keep(record: &LogRecord) -> bool { (record.severity >= 3 && !record.code.is_multiple_of(7)) || record.code == 777 }

fn to_bucket(record: &LogRecord, heavy: bool) -> Agg {
    let rounds = if heavy { 8 + record.payload_len as usize / 96 } else { 2 };
    let parsed = cpu_mix(rounds, record.payload_seed ^ record.ts ^ ((record.user_id as u64) << 17));
    let mut agg = Agg { checksum: parsed.wrapping_add(record.code as u64), ..Agg::default() };
    match record.severity { 4 | 5 => agg.err = 1, 3 => agg.warn = 1, _ => agg.info = 1 }
    agg
}

fn merge(a: Agg, b: Agg) -> Agg { Agg { err: a.err + b.err, warn: a.warn + b.warn, info: a.info + b.info, checksum: a.checksum.wrapping_add(b.checksum) } }

fn reduce_seq(input: &[LogRecord], heavy: bool) -> Option<Agg> { input.iter().filter(|record| keep(record)).map(|record| to_bucket(record, heavy)).reduce(merge) }
fn reduce_rayon(input: &[LogRecord], heavy: bool) -> Option<Agg> { input.par_iter().filter(|record| keep(record)).map(|record| to_bucket(record, heavy)).reduce_with(merge) }
fn reduce_orx(input: &[LogRecord], heavy: bool) -> Option<Agg> { input.par().filter(|record| keep(record)).map(|record| to_bucket(record, heavy)).reduce(merge) }

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = Vec<LogRecord>;
    type Output = Option<Agg>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input { inputs(input_variant.len()) }
    fn execute(&mut self, input_variant: &Self::InputFactors, alg_variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output {
        match alg_variant { Method::Seq => reduce_seq(input, input_variant.heavy), Method::Rayon => reduce_rayon(input, input_variant.heavy), Method::OrxOnce | Method::OrxBasic | Method::OrxRayon => reduce_orx(input, input_variant.heavy) }
    }
    fn expected_output(&self, input_variant: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> { Some(reduce_seq(input, input_variant.heavy)) }
}
