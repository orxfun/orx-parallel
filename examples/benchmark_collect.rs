use orx_parallel::{generic_iterator::GenericIterator, IntoParIter, ParIter};
use rayon::iter::IntoParallelIterator;
use std::{
    fmt::Debug,
    hint::black_box,
    time::{Duration, SystemTime},
};

fn initial_iterator(
    n: usize,
) -> GenericIterator<
    u64,
    impl Iterator<Item = u64>,
    impl rayon::iter::ParallelIterator<Item = u64>,
    impl ParIter<Item = u64>,
> {
    let vec = || (0..n as u64).collect::<Vec<_>>();
    let sequential = vec().into_iter();
    let rayon = vec().into_par_iter();
    let orx = vec().into_par();
    GenericIterator::new(sequential, rayon, orx)
}

fn sequential(
    iter: GenericIterator<
        String,
        impl Iterator<Item = String>,
        impl rayon::iter::ParallelIterator<Item = String>,
        impl ParIter<Item = String>,
    >,
) -> Vec<String> {
    iter.sequential().collect()
}

fn rayon(
    iter: GenericIterator<
        String,
        impl Iterator<Item = String>,
        impl rayon::iter::ParallelIterator<Item = String>,
        impl ParIter<Item = String>,
    >,
) -> Vec<String> {
    iter.rayon().collect()
}

fn orx(
    iter: GenericIterator<
        String,
        impl Iterator<Item = String>,
        impl rayon::iter::ParallelIterator<Item = String>,
        impl ParIter<Item = String>,
    >,
) -> Vec<String> {
    iter.orx().collect()
}

fn time<F, Out, O>(num_repetitions: usize, expected_output: Vec<O>, fun: F) -> Duration
where
    F: Fn() -> Out,
    Out: IntoIterator<Item = O>,
    O: PartialEq + Debug,
{
    let result = fun();
    assert_eq!(result.into_iter().collect::<Vec<_>>(), expected_output);

    // warm up
    for _ in 0..10 {
        let _ = black_box(fun());
    }

    // measurement

    let now = SystemTime::now();
    for _ in 0..num_repetitions {
        let _ = black_box(fun());
    }
    now.elapsed().unwrap()
}

fn main() {
    let n = 100000;
    let num_repetitions = 1000;

    let iter = || {
        initial_iterator(n)
            .map(|x| x.to_string())
            .filter_map(|x| (!x.starts_with('1')).then_some(x))
            .flat_map(|x| [format!("{}!", &x), x])
            .filter(|x| !x.starts_with('2'))
            .filter_map(|x| x.parse::<u64>().ok())
            .map(|x| x.to_string())
    };

    let elapsed = time(num_repetitions, sequential(iter()), || sequential(iter()));
    println!("sequential : {:?}", elapsed);

    let elapsed = time(num_repetitions, sequential(iter()), || rayon(iter()));
    println!("rayon      : {:?}", elapsed);

    let elapsed = time(num_repetitions, sequential(iter()), || orx(iter()));
    println!("orx        : {:?}", elapsed);
}
