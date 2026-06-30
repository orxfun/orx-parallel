use rand::{Rng, SeedableRng, rngs::SmallRng};

pub fn seed_for(seed: u64, k: usize) -> u64 {
    seed ^ (k as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

pub fn rng_from_seed(seed: u64) -> impl Rng {
    SmallRng::seed_from_u64(seed)
}
