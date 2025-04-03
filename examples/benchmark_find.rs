mod utils;

use clap::Parser;
use orx_parallel::{generic_iterator::GenericIterator, IntoParIter, ParIter};
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

enum FindWhen {
    Early,
    Middle,
    Late,
    Never,
}

fn get_find(n: usize, find_when: FindWhen) -> impl Fn(&String) -> bool {
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

fn reduce(a: usize, b: usize) -> usize {
    (a + b).saturating_sub(1)
}

// fn compute(
//     iter: GenericIterator<
//         usize,
//         impl Iterator<Item = usize>,
//         impl ParallelIterator<Item = usize>,
//         impl ParIter<Item = usize>,
//     >,
//     find: impl Fn(&String) -> bool,
// ) -> usize {
//     iter.map(|x| x.to_string())
//         .filter_map(|x| (!x.starts_with('1')).then_some(x))
//         .flat_map(|x| [format!("{}!", &x), x])
//         .filter(|x| !x.starts_with('2'))
//         .filter_map(|x| x.parse::<u64>().ok())
//         .find(find)
// }

fn main() {}
