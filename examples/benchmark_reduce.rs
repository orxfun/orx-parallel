use orx_parallel::{generic_iterator::GenericIterator, IntoParIter, ParIter};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    fmt::Debug,
    hint::black_box,
    time::{Duration, SystemTime},
};
use utils::timed_reduce_all;

mod utils;

// fn initial_iterator(
//     n: usize,
// ) -> GenericIterator<
//     u64,
//     impl Iterator<Item = u64>,
//     impl rayon::iter::ParallelIterator<Item = u64>,
//     impl ParIter<Item = u64>,
// > {
//     let vec = || (0..n as u64).collect::<Vec<_>>();
//     let sequential = vec().into_iter();
//     let rayon = vec().into_par_iter();
//     let orx = vec().into_par();
//     GenericIterator::new(sequential, rayon, orx)
// }

// fn sequential(
//     iter: GenericIterator<
//         usize,
//         impl Iterator<Item = usize>,
//         impl rayon::iter::ParallelIterator<Item = usize>,
//         impl ParIter<Item = usize>,
//     >,
// ) -> usize {
//     iter.sequential().sum()
// }

// fn rayon(
//     iter: GenericIterator<
//         usize,
//         impl Iterator<Item = usize>,
//         impl rayon::iter::ParallelIterator<Item = usize>,
//         impl ParIter<Item = usize>,
//     >,
// ) -> usize {
//     iter.rayon().sum()
// }

// fn orx(
//     iter: GenericIterator<
//         usize,
//         impl Iterator<Item = usize>,
//         impl rayon::iter::ParallelIterator<Item = usize>,
//         impl ParIter<Item = usize>,
//     >,
// ) -> usize {
//     iter.orx().sum()
// }

// fn time<F, O>(num_repetitions: usize, expected_output: O, fun: F) -> Duration
// where
//     F: Fn() -> O,
//     O: PartialEq + Debug,
// {
//     let result = fun();
//     assert_eq!(result, expected_output);

//     // warm up
//     for _ in 0..10 {
//         let _ = black_box(fun());
//     }

//     // measurement

//     let now = SystemTime::now();
//     for _ in 0..num_repetitions {
//         let _ = black_box(fun());
//     }
//     now.elapsed().unwrap()
// }

fn compute(
    iter: GenericIterator<
        usize,
        impl Iterator<Item = usize>,
        impl ParallelIterator<Item = usize>,
        impl ParIter<Item = usize>,
    >,
) -> usize {
    0
}

fn main() {
    let n = 100000;
    let num_repetitions = 1000;

    let iter = move || {
        (0..n as u64)
            .collect::<Vec<_>>()
            .into_iter()
            .map(|x| x.to_string())
            .filter_map(|x| (!x.starts_with('1')).then_some(x))
            .flat_map(|x| [format!("{}!", &x), x])
            .filter(|x| !x.starts_with('2'))
            .filter_map(|x| x.parse::<u64>().ok())
            .map(|x| x.to_string().len())
            .collect::<Vec<_>>()
    };

    let expected_output = iter().into_iter().sum();

    let computations: Vec<(&str, Box<dyn Fn() -> usize>)> = vec![
        (
            "sequential",
            Box::new(move || compute(GenericIterator::sequential(iter().into_iter()))),
        ),
        (
            "rayon",
            Box::new(move || compute(GenericIterator::rayon(iter().into_par_iter()))),
        ),
        (
            "orx",
            Box::new(move || compute(GenericIterator::orx(iter().into_par()))),
        ),
    ];

    timed_reduce_all(num_repetitions, expected_output, &computations);
}
