use orx_parallel::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::sync::atomic::{AtomicUsize, Ordering};

const N: u64 = 500_000;

fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn sequential() -> u64 {
    let mut rng = ChaCha20Rng::seed_from_u64(42);

    let input: Vec<u64> = (1..N).collect();

    input
        .into_iter()
        .map(|i| fibonacci(i) % 1000 + 1)
        .map(|i| rng.random_range(0..i))
        .sum()
}

fn par_using() -> u64 {
    let rng = ChaCha20Rng::seed_from_u64(42);

    let input: Vec<u64> = (1..N).collect();

    input
        .into_par()
        .using(rng)
        .map(|_, i| fibonacci(i) % 1000 + 1)
        .map(|rng: &mut ChaCha20Rng, i: u64| rng.random_range(0..i))
        .sum()
}

fn par_using_counting_clones() -> u64 {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    struct Rng(ChaCha20Rng);
    unsafe impl Send for Rng {}
    impl Clone for Rng {
        fn clone(&self) -> Self {
            _ = COUNTER.fetch_add(1, Ordering::Relaxed);
            Self(self.0.clone())
        }
    }

    let rng = ChaCha20Rng::seed_from_u64(42);

    let input: Vec<u64> = (1..N).collect();

    let result = input
        .into_par()
        .num_threads(8)
        .using(Rng(rng))
        .map(|_, i| fibonacci(i) % 1000 + 1)
        .map(|rng: &mut Rng, i: u64| rng.0.random_range(0..i))
        .sum();

    let num_clones = COUNTER.load(Ordering::Relaxed);
    println!("> number of clones (= used threads) = {num_clones}");

    result
}

fn main() {
    println!("\n\nsequential result:");
    let sequential_result = sequential();
    println!("{sequential_result}");

    println!("\n\nparallel result:");
    let parallel_result = par_using();
    println!("{parallel_result}");

    println!("\n\nparallel result counting clones:");
    let parallel_result = par_using_counting_clones();
    println!("{parallel_result}");
}
