use orx_parallel::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::sync::atomic::{AtomicUsize, Ordering};

const N: u64 = 100_000;

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

fn using() -> u64 {
    let input: Vec<u64> = (1..N).collect();

    input
        .into_par()
        .using(|thread_idx| ChaCha20Rng::seed_from_u64(thread_idx as u64 * 10))
        .map(|_, i| fibonacci(i) % 1000 + 1)
        .filter(|rng: &mut ChaCha20Rng, _: &u64| rng.random_bool(0.4))
        .map(|rng: &mut ChaCha20Rng, i: u64| rng.random_range(0..i))
        .sum()
}

fn using_clone() -> u64 {
    let rng = ChaCha20Rng::seed_from_u64(42);

    let input: Vec<u64> = (1..N).collect();

    input
        .into_par()
        .using_clone(rng)
        .map(|_, i| fibonacci(i) % 1000 + 1)
        .filter(|rng: &mut ChaCha20Rng, _: &u64| rng.random_bool(0.4))
        .map(|rng: &mut ChaCha20Rng, i: u64| rng.random_range(0..i))
        .sum()
}

fn using_clone_while_counting_clones() -> u64 {
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
        .using_clone(Rng(rng))
        .map(|_, i| fibonacci(i) % 1000 + 1)
        .filter(|rng: &mut Rng, _: &u64| rng.0.random_bool(0.4))
        .map(|rng: &mut Rng, i: u64| rng.0.random_range(0..i))
        .sum();

    let num_clones = COUNTER.load(Ordering::Relaxed);
    println!("\n> number of times RNG is cloned (= used threads) = {num_clones}");

    result
}

fn main() {
    println!("\n\n\n### I. using:");
    println!(
        "* Allows to create the random number generator (RNG) of each thread using a closure that takes the thread index."
    );
    println!("* Hence, we can guarantee to use different RNGs for each thread.");
    println!("* The following is the example 'using' transformation used in this example.");
    println!("\n  >  par.using(|thread_idx| ChaCha20Rng::seed_from_u64(thread_idx as u64 * 10))");
    let parallel_result = using();
    println!("\n* result = {parallel_result}");

    println!("\n\n\n### II. using_clone:");
    println!(
        "* With this call we create and pass in the RNG; then each thread receives a clone of it as in its initial state."
    );
    println!("\n  >  par.using_clone(my_rng)");
    let parallel_result = using_clone();
    println!("\n* result = {parallel_result}");

    println!("\n\n\n### III. using_clone with a hook to count number of times the RNG is cloned:");
    println!(
        "* The computation is limited to 8 threads; therefore, at most, 8 clones of the RNG must be created."
    );
    println!(
        "* This example demonstrates that exactly one value is created-or-cloned per thread used."
    );
    let parallel_result = using_clone_while_counting_clones();
    println!("{parallel_result}");
    println!("\n* result = {parallel_result}");
}
