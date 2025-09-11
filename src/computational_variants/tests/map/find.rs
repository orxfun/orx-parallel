use crate::{
    Params, computations::map_self, orch::DefaultOrchestrator, runner::parallel_runner_compute,
};
use orx_concurrent_iter::IntoConcurrentIter;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64])
]
fn m_find(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();

    let expected = input.clone().into_iter().next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();

    let output =
        parallel_runner_compute::next::m(DefaultOrchestrator::default(), params, iter, map_self).1;
    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64])
]
fn m_map_find(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map = |x: String| format!("{}!", x);

    let expected = input.clone().into_iter().map(map).next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let output =
        parallel_runner_compute::next::m(DefaultOrchestrator::default(), params, iter, map).1;

    assert_eq!(expected, output);
}
