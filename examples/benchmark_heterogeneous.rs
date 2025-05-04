mod utils;

#[cfg(not(feature = "generic_iterator"))]
fn main() {
    panic!(
        r#"

REQUIRES FEATURE: generic_iterator

To view the arguments:
cargo run --release --features generic_iterator --example benchmark_heterogeneous -- --help

To run with default arguments:
cargo run --release --features generic_iterator --example benchmark_heterogeneous

To run with desired arguments:
cargo run --release --features generic_iterator --example benchmark_heterogeneous -- --len 123456 --num-repetitions 10

Play with the transformations inside the compute method to test out different computations.

"#
    );
}

#[cfg(feature = "generic_iterator")]
fn main() {
    use clap::Parser;
    use orx_parallel::{IntoParIter, ParIter, generic_iterator::GenericIterator};
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use utils::timed_reduce_all;

    #[derive(Parser, Debug)]
    struct Args {
        /// Number of items in the input iterator.
        #[arg(long, default_value_t = 2000)]
        len: usize,
        /// Number of repetitions to measure time; total time will be reported.
        #[arg(long, default_value_t = 100)]
        num_repetitions: usize,
    }

    fn fibonacci(n: &u64) -> u64 {
        let mut a = 0;
        let mut b = 1;
        for _ in 0..*n {
            let c = a + b;
            a = b;
            b = c;
        }
        a
    }

    fn heterogeneous_computation(i: usize) -> u64 {
        let mut rng = ChaCha8Rng::seed_from_u64(i as u64);
        for _ in 0..10 * i {
            let _: u32 = rng.random();
        }
        let n = match rng.random_bool(0.75) {
            true => rng.random_range(1..100),
            false => rng.random_range(10000..20000),
        };
        fibonacci(&n)
    }

    fn compute(
        iter: GenericIterator<
            usize,
            impl Iterator<Item = usize>,
            impl ParallelIterator<Item = usize>,
            impl ParIter<Item = usize>,
        >,
    ) -> u64 {
        iter.map(heterogeneous_computation).max().unwrap()
    }

    let args = Args::parse();

    let input = move || (0..args.len as usize).collect::<Vec<_>>();
    let expected_output = compute(GenericIterator::sequential(input().into_iter()));

    let computations: Vec<(&str, Box<dyn Fn() -> u64>)> = vec![
        (
            "sequential",
            Box::new(move || compute(GenericIterator::sequential(input().into_iter()))),
        ),
        (
            "rayon",
            Box::new(move || compute(GenericIterator::rayon(input().into_par_iter()))),
        ),
        (
            "orx",
            Box::new(move || compute(GenericIterator::orx(input().into_par()))),
        ),
    ];

    timed_reduce_all(
        "benchmark_heterogeneous",
        args.num_repetitions,
        Some(expected_output),
        &computations,
    );
}
