/*
1. to run the computation without any limits on max number of threads:
cargo run --release --example max_num_threads_config
OR
ORX_PARALLEL_MAX_NUM_THREADS=0 cargo run --release --example max_num_threads_config

2. to allow parallel computation at most 4 threads:
ORX_PARALLEL_MAX_NUM_THREADS=4 cargo run --release --example max_num_threads_config
*/

use orx_parallel::*;

fn fib(n: &u64) -> u64 {
    // just some work
    let mut a = 0;
    let mut b = 1;
    for _ in 0..*n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

// A: what should name of the variable be?
const MAX_NUM_THREADS_ENV_VARIABLE: &str = "ORX_PARALLEL_MAX_NUM_THREADS";

fn max_num_threads_by_env_variable() -> Option<usize> {
    match std::env::var(MAX_NUM_THREADS_ENV_VARIABLE) {
        Ok(s) => match s.parse::<usize>() {
            Ok(0) => None,
            Ok(x) => Some(x),
            Err(_e) => None,
        },
        Err(_e) => None,
    }
}

fn main() {
    match max_num_threads_by_env_variable() {
        Some(x) => {
            println!(
                "Environment variable {MAX_NUM_THREADS_ENV_VARIABLE} is set to {x}\n -> this will be the hard limit on the maximum number of threads that can be used by parallelization"
            )
        }
        None => {
            println!(
                "Environment variable {MAX_NUM_THREADS_ENV_VARIABLE} is not set\n -> all available threads might be used"
            )
        }
    }

    let n = 1 << 31;
    let input = 0..n;

    // default -> might use all threads
    let sum = input.par().map(|i| fib(&(i as u64 % 42)) % 42).sum();
    println!("sum = {sum}");
}
