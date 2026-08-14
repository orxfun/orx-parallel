use crate::{
    alg::Method,
    input::{Dist, InputVariant},
};
use bench_helper::runner::cpu_mix;
use orx_criterion::Experiment;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

const KEY_SPACE: usize = 4096;
const TOP_K: usize = 32;
const CPU_MIX_ROUNDS: usize = 40;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Agg {
    pub total: u64,
    pub top_sum: u64,
    pub checksum: u64,
}

pub struct Exp;

fn inputs(len: usize, dist: Dist) -> Vec<u16> {
    const SEED: u64 = 0xFACE_CAFE_1357_2468;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);

    (0..len)
        .map(|_| {
            let key = match dist {
                Dist::Uniform => rng.random_range(0..KEY_SPACE as u64),
                Dist::Skewed => {
                    let u = rng.random::<f64>();
                    let x = (u * u * u * u) * KEY_SPACE as f64;
                    x as u64
                }
            }
            .min((KEY_SPACE - 1) as u64);
            key as u16
        })
        .collect()
}

fn count_seq(input: &[u16]) -> Vec<u64> {
    let mut counts = vec![0u64; KEY_SPACE];
    for key in input {
        counts[*key as usize] += cpu_mix(CPU_MIX_ROUNDS, *key as u64);
    }
    counts
}

fn merge_counts(mut a: Vec<u64>, b: Vec<u64>) -> Vec<u64> {
    for (x, y) in a.iter_mut().zip(b) {
        *x += y;
    }
    a
}

fn count_rayon(input: &[u16]) -> Vec<u64> {
    input
        .par_iter()
        .fold(
            || vec![0u64; KEY_SPACE],
            |mut local, key| {
                local[*key as usize] += cpu_mix(CPU_MIX_ROUNDS, *key as u64);
                local
            },
        )
        .reduce(|| vec![0u64; KEY_SPACE], merge_counts)
}

fn count_orx(input: &[u16]) -> Vec<u64> {
    let mut use_vec = UseVec::new(|_| vec![0u64; KEY_SPACE]);

    input
        .into_par()
        .use_vec(&mut use_vec)
        .for_each(|local, key| {
            local[*key as usize] += cpu_mix(CPU_MIX_ROUNDS, *key as u64);
        });

    use_vec
        .into_vec()
        .into_iter()
        .reduce(merge_counts)
        .unwrap_or_else(|| vec![0u64; KEY_SPACE])
}

fn topk_agg(counts: &[u64]) -> Agg {
    let mut entries: Vec<(usize, u64)> = counts.iter().copied().enumerate().collect();
    entries.sort_unstable_by(|(ka, ca), (kb, cb)| cb.cmp(ca).then(ka.cmp(kb)));

    let mut checksum = 0_u64;
    let mut top_sum = 0_u64;

    for (rank, (key, count)) in entries.into_iter().take(TOP_K).enumerate() {
        top_sum += count;
        checksum ^= ((key as u64) << 20) ^ (count << 7) ^ rank as u64;
    }

    Agg {
        total: counts.iter().sum(),
        top_sum,
        checksum,
    }
}

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = Vec<u16>;
    type Output = Agg;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(input_variant.len(), input_variant.dist)
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let counts = match alg_variant {
            Method::Seq => count_seq(input),
            Method::Rayon => count_rayon(input),
            Method::OrxOnce => count_orx(input),
            Method::OrxBasic => count_orx(input),
            Method::OrxRayon => count_orx(input),
        };

        topk_agg(&counts)
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(topk_agg(&count_seq(input)))
    }
}
