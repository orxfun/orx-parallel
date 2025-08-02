use orx_parallel::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

const N: usize = 1_000_000;

fn sequential() -> usize {
    let mut rng = ChaCha20Rng::seed_from_u64(42);

    let input: Vec<_> = (1..N).collect();

    input.into_iter().map(|i| rng.random_range(0..i)).sum()
}

// fn par_using() -> usize {
//     let mut rng = ChaCha20Rng::seed_from_u64(42);

//     let input: Vec<_> = (1..N).collect();

//     input.into_par().map(|i| rng.random_range(0..i)).sum()
// }

fn main() {
    let sequential_result = sequential();
    println!("sequential result = {sequential_result}");
}
