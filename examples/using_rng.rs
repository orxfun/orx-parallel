use orx_parallel::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

const N: u64 = 1_000_000;

fn sequential() -> u64 {
    let mut rng = ChaCha20Rng::seed_from_u64(42);

    let input: Vec<u64> = (1..N).collect();

    input.into_iter().map(|i| rng.random_range(0..i)).sum()
}

fn par_using() -> u64 {
    let rng = ChaCha20Rng::seed_from_u64(42);

    let input: Vec<u64> = (1..N).collect();

    input
        .into_par()
        .using(rng)
        .map_u(|rng: &mut ChaCha20Rng, i: u64| rng.random_range(0..i))
        .sum()
}

fn main() {
    let sequential_result = sequential();
    println!("sequential result = {sequential_result}");

    let parallel_result = par_using();
    println!("parallel result = {parallel_result}");
}
