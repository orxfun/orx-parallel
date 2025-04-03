use std::{
    fmt::Debug,
    hint::black_box,
    time::{Duration, SystemTime},
};

fn timed_reduce<F, O>(num_repetitions: usize, expected_output: O, fun: F) -> Duration
where
    F: Fn() -> O,
    O: PartialEq + Debug,
{
    let result = fun();
    assert_eq!(result, expected_output);

    // warm up
    for _ in 0..10 {
        let _ = black_box(fun());
    }

    // measurement

    let now = SystemTime::now();
    for _ in 0..num_repetitions {
        let result = black_box(fun());
        assert_eq!(result, expected_output);
    }
    now.elapsed().unwrap()
}

pub fn timed_reduce_all<O>(
    num_repetitions: usize,
    expected_output: O,
    computations: &[(&str, Box<dyn Fn() -> O>)],
) where
    O: PartialEq + Debug + Copy,
{
    for (name, fun) in computations {
        let duration = timed_reduce(num_repetitions, expected_output, fun);
        println!("{:>10} : {:?}", name, duration);
    }
}
