mod utils;

#[cfg(not(feature = "generic_iterator"))]
fn main() {
    panic!(
        r#"

REQUIRES FEATURE: generic_iterator

To view the arguments:
cargo run --release --features generic_iterator --example benchmark_collect -- --help

To run with default arguments:
cargo run --release --features generic_iterator --example benchmark_collect

To run with desired arguments:
cargo run --release --features generic_iterator --example benchmark_collect -- --len 123456 --num-repetitions 10

Play with the transformations inside the compute method to test out different computations.

"#
    );
}

#[cfg(feature = "generic_iterator")]
fn main() {
    use clap::Parser;
    use orx_parallel::{IntoParIter, ParIter, generic_iterator::GenericIterator};
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use utils::timed_collect_all;

    #[derive(Parser, Debug)]
    struct Args {
        /// Number of items in the input iterator.
        #[arg(long, default_value_t = 100000)]
        len: usize,
        /// Number of repetitions to measure time; total time will be reported.
        #[arg(long, default_value_t = 100)]
        num_repetitions: usize,
    }

    fn compute(
        iter: GenericIterator<
            usize,
            impl Iterator<Item = usize>,
            impl ParallelIterator<Item = usize>,
            impl ParIter<Item = usize>,
        >,
    ) -> Vec<String> {
        iter.map(|x| x.to_string())
            .filter_map(|x| (!x.starts_with('1')).then_some(x))
            .flat_map(|x| [format!("{}!", &x), x])
            .filter(|x| !x.starts_with('2'))
            .filter_map(|x| x.parse::<u64>().ok())
            .map(|x| x.to_string())
            .collect_vec()
    }

    let args = Args::parse();

    let input = move || (0..args.len as usize).collect::<Vec<_>>();
    let expected_output = compute(GenericIterator::sequential(input().into_iter()));

    let computations: Vec<(&str, Box<dyn Fn() -> Vec<String>>)> = vec![
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

    timed_collect_all(
        "benchmark_collect",
        args.num_repetitions,
        &expected_output,
        &computations,
    );
}
