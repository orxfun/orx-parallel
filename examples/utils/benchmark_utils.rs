#![allow(dead_code)]

use std::{
    fmt::Debug,
    hint::black_box,
    time::{Duration, SystemTime},
};

// reduce

fn timed_reduce<F, O>(num_repetitions: usize, expected_output: &Option<O>, fun: F) -> Duration
where
    F: Fn() -> O,
    O: PartialEq + Debug,
{
    if let Some(expected_output) = expected_output.as_ref() {
        let result = fun();
        assert_eq!(&result, expected_output);
    }

    // warm up
    for _ in 0..10 {
        let _ = black_box(fun());
    }

    // measurement

    let now = SystemTime::now();
    for _ in 0..num_repetitions {
        let result = black_box(fun());
        if let Some(expected_output) = expected_output.as_ref() {
            assert_eq!(&result, expected_output);
        }
    }
    now.elapsed().unwrap()
}

pub fn timed_reduce_all<O>(
    benchmark_name: &str,
    num_repetitions: usize,
    expected_output: Option<O>,
    computations: &[(&str, Box<dyn Fn() -> O>)],
) where
    O: PartialEq + Debug + Clone,
{
    println!("\n{} {} {}", "#".repeat(10), benchmark_name, "#".repeat(10));
    for (name, fun) in computations {
        let duration = timed_reduce(num_repetitions, &expected_output, fun);
        println!("{:>10} : {:?}", name, duration);
    }
    println!("{}\n", "#".repeat(10 + 10 + 2 + benchmark_name.len()));
}

// collect

fn timed_collect<F, Out, O>(num_repetitions: usize, expected_output: &[O], fun: F) -> Duration
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

pub fn timed_collect_all<Out, O>(
    benchmark_name: &str,
    num_repetitions: usize,
    expected_output: &[O],
    computations: &[(&str, Box<dyn Fn() -> Out>)],
) where
    Out: IntoIterator<Item = O>,
    O: PartialEq + Debug,
{
    println!("\n{} {} {}", "#".repeat(10), benchmark_name, "#".repeat(10));
    for (name, fun) in computations {
        let duration = timed_collect(num_repetitions, expected_output, fun);
        println!("{:>10} : {:?}", name, duration);
    }
    println!("{}\n", "#".repeat(10 + 10 + 2 + benchmark_name.len()));
}
