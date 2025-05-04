mod utils;

#[cfg(not(feature = "generic_iterator"))]
fn main() {
    panic!(
        r#"

REQUIRES FEATURE: generic_iterator

To view the arguments:
cargo run --release --features generic_iterator --example benchmark_find -- --help

To run with default arguments:
cargo run --release --features generic_iterator --example benchmark_find

To run with desired arguments:
cargo run --release --features generic_iterator --example benchmark_find -- --len 123456 --num-repetitions 10

Play with the transformations inside the compute method to test out different computations.

"#
    );
}

#[cfg(feature = "generic_iterator")]
fn main() {
    use clap::Parser;
    use orx_parallel::{generic_iterator::GenericIterator, IntoParIter, ParIter};
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use std::fmt::Display;
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

    #[derive(Parser, Debug, Clone, Copy)]
    enum FindWhen {
        Early,
        Middle,
        Late,
        Never,
    }

    impl Display for FindWhen {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                FindWhen::Early => write!(f, "at the BEGINNING of the iterator"),
                FindWhen::Middle => write!(f, "in the MIDDLE of the iterator"),
                FindWhen::Late => write!(f, "at the END of the iterator"),
                FindWhen::Never => write!(f, "is NOT in the iterator"),
            }
        }
    }

    fn get_find(n: usize, find_when: FindWhen) -> impl Fn(&String) -> bool + Send + Sync + Clone {
        move |x| match find_when {
            FindWhen::Early => x.starts_with("3"),
            FindWhen::Middle => {
                let position = n / 2;
                x == &position.to_string()
            }
            FindWhen::Late => {
                let position = n.saturating_sub(1);
                x == &position.to_string()
            }
            FindWhen::Never => x.starts_with("x"),
        }
    }

    fn compute(
        find: impl Fn(&String) -> bool + Send + Sync + Clone,
        iter: GenericIterator<
            usize,
            impl Iterator<Item = usize>,
            impl ParallelIterator<Item = usize>,
            impl ParIter<Item = usize>,
        >,
    ) -> String {
        iter.map(|x| x.to_string())
            .filter_map(|x| (!x.starts_with('1')).then_some(x))
            .flat_map(|x| [format!("{}!", &x), x])
            .filter(|x| !x.starts_with('2'))
            .filter_map(|x| (!x.ends_with("!")).then_some(x))
            .find(find)
            .unwrap_or_default()
    }

    let args = Args::parse();
    let find_when = [
        FindWhen::Early,
        FindWhen::Middle,
        FindWhen::Late,
        FindWhen::Never,
    ];

    let input = move || (0..args.len as usize).collect::<Vec<_>>();

    for when in find_when {
        let find = move || get_find(args.len, when);
        let expected_output = compute(find(), GenericIterator::sequential(input().into_iter()));

        let computations: Vec<(&str, Box<dyn Fn() -> String>)> = vec![
            (
                "sequential",
                Box::new(move || compute(find(), GenericIterator::sequential(input().into_iter()))),
            ),
            (
                "rayon",
                Box::new(move || compute(find(), GenericIterator::rayon(input().into_par_iter()))),
            ),
            (
                "orx",
                Box::new(move || compute(find(), GenericIterator::orx(input().into_par()))),
            ),
        ];

        timed_reduce_all(
            &format!("find item that is {}", when),
            args.num_repetitions,
            Some(expected_output),
            &computations,
        );
    }
}
