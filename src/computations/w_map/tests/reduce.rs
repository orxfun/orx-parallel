use crate::{Params, computations::w_map::m::WithM, runner::DefaultRunner};
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
fn m_reduce(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let reduce = |x: String, y: String| match x < y {
        true => y,
        false => x,
    };

    let expected = input.clone().into_iter().reduce(reduce);

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let map = |counter: &mut usize, x| {
        *counter += 1;
        x
    };
    let m = WithM::new(params, iter, 0, map);
    let (_, output) = m.reduce::<DefaultRunner, _>(reduce);

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn m_map_reduce(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map = |counter: &mut usize, x: String| {
        *counter += 1;
        format!("{}!", x)
    };
    let reduce = |x: String, y: String| match x < y {
        true => y,
        false => x,
    };

    let mut counter = 0;
    let expected = input
        .clone()
        .into_iter()
        .map(|x| map(&mut counter, x))
        .reduce(reduce);

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let m = WithM::new(params, iter, 0, map);
    let (_, output) = m.reduce::<DefaultRunner, _>(reduce);

    assert_eq!(expected, output);
}
