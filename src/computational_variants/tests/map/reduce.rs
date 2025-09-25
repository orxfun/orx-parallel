use crate::{Params, default_fns::map_self, executor::parallel_compute, runner::DefaultRunner};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
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
fn m_reduce(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let reduce = |x: String, y: String| match x < y {
        true => y,
        false => x,
    };

    let expected = input.clone().into_iter().reduce(reduce);

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let (_, output) =
        parallel_compute::reduce::m(DefaultRunner::default(), params, iter, map_self, reduce);

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64])
]
fn m_map_reduce(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map = |x: String| format!("{}!", x);
    let reduce = |x: String, y: String| match x < y {
        true => y,
        false => x,
    };

    let expected = input.clone().into_iter().map(map).reduce(reduce);

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let (_, output) =
        parallel_compute::reduce::m(DefaultRunner::default(), params, iter, map, reduce);

    assert_eq!(expected, output);
}
