//! Arbitrary-iterator benchmark for HashMap/HashSet processing via iterators.
//! Compares sequential iteration with rayon bridge and orx `iter_into_par`
//! pipelines over map/filter/reduce workloads.

use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
enum Dataset {
    Map,
    Set,
}

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
    dataset: Dataset,
}

impl InputVariant {
    fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "dataset"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.dataset {
                Dataset::Map => "hash-map",
                Dataset::Set => "hash-set",
            }
            .to_string(),
        ]
    }
}

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Agg {
    sum: u64,
    count: u64,
}

impl Agg {
    fn from_val(v: u64) -> Self {
        Self { sum: v, count: 1 }
    }
}

fn merge(a: Agg, b: Agg) -> Agg {
    Agg {
        sum: a.sum.wrapping_add(b.sum),
        count: a.count + b.count,
    }
}

#[derive(Clone, Debug)]
struct Input {
    map: HashMap<u64, u32>,
    set: HashSet<u64>,
}

struct Exp;

fn mix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn keep(v: u64) -> bool {
    v & 0x7 != 0 && !v.trailing_zeros().is_multiple_of(3)
}

fn inputs(len: usize) -> Input {
    const SEED: u64 = 0xA5A5_4F4F_0101_BEEF;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);

    let mut map = HashMap::with_capacity(len);
    let mut set = HashSet::with_capacity(len);

    for idx in 0..len {
        let key_map = (idx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x1234_5678_90AB_CDEF;
        let value = rng.random_range(1..=1_000_000);
        map.insert(key_map, value);

        let key_set = (idx as u64).wrapping_mul(0xD6E8_FD9D_79A1_4E3B) ^ 0x0FED_CBA9_8765_4321;
        set.insert(key_set);
    }

    Input { map, set }
}

fn seq_map(map: &HashMap<u64, u32>) -> Agg {
    map.iter()
        .map(|(k, v)| mix64(*k ^ (*v as u64).rotate_left(13)))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

fn seq_set(set: &HashSet<u64>) -> Agg {
    set.iter()
        .map(|k| mix64(*k ^ 0xF0F0_0F0F_AAAA_5555))
        .filter(|x| keep(*x))
        .map(Agg::from_val)
        .reduce(merge)
        .unwrap_or_default()
}

fn rayon_map(map: &HashMap<u64, u32>, num_threads: usize) -> Agg {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        map.iter()
            .par_bridge()
            .map(|(k, v)| mix64(*k ^ (*v as u64).rotate_left(13)))
            .filter(|x| keep(*x))
            .map(Agg::from_val)
            .reduce(Agg::default, merge)
    })
}

fn rayon_set(set: &HashSet<u64>, num_threads: usize) -> Agg {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        set.iter()
            .par_bridge()
            .map(|k| mix64(*k ^ 0xF0F0_0F0F_AAAA_5555))
            .filter(|x| keep(*x))
            .map(Agg::from_val)
            .reduce(Agg::default, merge)
    })
}

fn orx_map(map: &HashMap<u64, u32>, fixed_runner: bool, num_threads: usize) -> Agg {
    let par = map
        .iter()
        .iter_into_par()
        .num_threads(num_threads)
        .map(|(k, v)| mix64(*k ^ (*v as u64).rotate_left(13)))
        .filter(|x| keep(*x))
        .map(Agg::from_val);

    let result = match fixed_runner {
        false => par.reduce(merge),
        true => par
            .runner(Runner::fixed(Pool::default(num_threads)))
            .reduce(merge),
    };

    result.unwrap_or_default()
}

fn orx_set(set: &HashSet<u64>, fixed_runner: bool, num_threads: usize) -> Agg {
    let par = set
        .iter()
        .iter_into_par()
        .num_threads(num_threads)
        .map(|k| mix64(*k ^ 0xF0F0_0F0F_AAAA_5555))
        .filter(|x| keep(*x))
        .map(Agg::from_val);

    let result = match fixed_runner {
        false => par.reduce(merge),
        true => par
            .runner(Runner::fixed(Pool::default(num_threads)))
            .reduce(merge),
    };

    result.unwrap_or_default()
}

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = Input;

    type Output = Agg;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(input_variant.len())
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match (input_variant.dataset, alg_variant) {
            (Dataset::Map, Method::Seq) => seq_map(&input.map),
            (Dataset::Map, Method::Rayon { nt }) => rayon_map(&input.map, *nt),
            (Dataset::Map, Method::Orx { nt }) => orx_map(&input.map, false, *nt),
            (Dataset::Map, Method::OrxFixed { nt }) => orx_map(&input.map, true, *nt),

            (Dataset::Set, Method::Seq) => seq_set(&input.set),
            (Dataset::Set, Method::Rayon { nt }) => rayon_set(&input.set, *nt),
            (Dataset::Set, Method::Orx { nt }) => orx_set(&input.set, false, *nt),
            (Dataset::Set, Method::OrxFixed { nt }) => orx_set(&input.set, true, *nt),
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        Some(match input_variant.dataset {
            Dataset::Map => seq_map(&input.map),
            Dataset::Set => seq_set(&input.set),
        })
    }
}

fn run(c: &mut Criterion) {
    let ns = [16, 20];
    let datasets = [Dataset::Map, Dataset::Set];
    let treatments: Vec<_> = ns
        .into_iter()
        .flat_map(|n| datasets.map(|dataset| InputVariant { n, dataset }))
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
        "arbitrary_iter_map_set_processing",
        &treatments,
        &variants,
    );
}

criterion_group!(benches, run);
criterion_main!(benches);
