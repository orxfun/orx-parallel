mod utils;

#[cfg(not(feature = "generic_iterator"))]
fn main() {
    panic!(
        r#"

REQUIRES FEATURE: generic_iterator

To view the arguments:
cargo run --release --features generic_iterator --example benchmark_reduce -- --help

To run with default arguments:
cargo run --release --features generic_iterator --example benchmark_reduce

To run with desired arguments:
cargo run --release --features generic_iterator --example benchmark_reduce -- --len 123456 --num-repetitions 10

Play with the transformations inside the compute method to test out different computations.

"#
    );
}

#[cfg(feature = "generic_iterator")]
fn main() {
    use clap::Parser;
    use orx_parallel::{IntoParIter, ParIter, generic_iterator::GenericIterator};
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use utils::timed_reduce_all;

    #[derive(Parser, Debug)]
    struct Args {
        /// Number of items in the input iterator.
        #[arg(long, default_value_t = 100000)]
        len: usize,
        /// Number of repetitions to measure time; total time will be reported.
        #[arg(long, default_value_t = 100)]
        num_repetitions: usize,
    }

    fn reduce(a: usize, b: usize) -> usize {
        (a + b).saturating_sub(1)
    }

    fn compute(
        iter: GenericIterator<
            usize,
            impl Iterator<Item = usize>,
            impl ParallelIterator<Item = usize>,
            impl ParIter<Item = usize>,
        >,
    ) -> usize {
        iter.map(|x| x.to_string())
            .filter_map(|x| (!x.starts_with('1')).then_some(x))
            .flat_map(|x| [format!("{}!", &x), x])
            .filter(|x| !x.starts_with('2'))
            .filter_map(|x| x.parse::<u64>().ok())
            .map(|x| x.to_string().len())
            .reduce(reduce)
            .unwrap_or(0)
    }

    let args = Args::parse();

    let input = move || (0..args.len as usize).collect::<Vec<_>>();
    let expected_output = compute(GenericIterator::sequential(input().into_iter()));

    let computations: Vec<(&str, Box<dyn Fn() -> usize>)> = vec![
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
        "benchmark_reduce",
        args.num_repetitions,
        Some(expected_output),
        &computations,
    );
}
