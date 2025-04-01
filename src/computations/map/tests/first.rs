use crate::{
    computations::{map::m::M, map_self},
    Params,
};
use orx_concurrent_iter::IntoConcurrentIter;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn m_first(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();

    let expected = input.clone().into_iter().next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let m = M::new(params, iter, map_self);
    let output = m.next();

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn m_map_first(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map = |x: String| format!("{}!", x);

    let expected = input.clone().into_iter().map(map).next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let m = M::new(params, iter, map);
    let output = m.next();

    assert_eq!(expected, output);
}
