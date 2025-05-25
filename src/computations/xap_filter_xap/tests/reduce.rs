use crate::computations::{Atom, Values, map_self_atom};
use crate::{Params, computations::xap_filter_xap::xfx::Xfx, runner::DefaultRunner};
use orx_concurrent_iter::IntoConcurrentIter;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn xfx_map_filter_reduce(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map1 = |x: String| Atom(format!("{}!", x));
    let filter = move |x: &String| match actual_filter {
        true => !x.starts_with('1'),
        false => true,
    };
    let reduce = |x: String, y: String| match x > y {
        true => x,
        false => y,
    };

    let expected = input
        .clone()
        .into_iter()
        .map(|x| map1(x).first().unwrap())
        .filter(filter)
        .reduce(reduce);

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let mfm = Xfx::new(params, iter, map1, filter, map_self_atom);

    let (_, output) = mfm.reduce::<DefaultRunner, _>(reduce);

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn xfx_filter_reduce(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let filter = move |x: &String| match actual_filter {
        true => !x.starts_with('1'),
        false => true,
    };
    let reduce = |x: String, y: String| match x > y {
        true => x,
        false => y,
    };

    let expected = input.clone().into_iter().filter(filter).reduce(reduce);

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let mfm = Xfx::new(params, iter, map_self_atom, filter, map_self_atom);

    let (_, output) = mfm.reduce::<DefaultRunner, _>(reduce);

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn xfx_map_filter_map_reduce(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map1 = |x: String| Atom(format!("{}!", x));
    let filter = move |x: &String| match actual_filter {
        true => !x.starts_with('1'),
        false => true,
    };
    let map2 = |x: String| Atom(x.len());
    let reduce = |x: usize, y: usize| x + y;

    let expected = input
        .clone()
        .into_iter()
        .map(|x| map1(x).first().unwrap())
        .filter(filter)
        .map(|x| map2(x).first().unwrap())
        .reduce(reduce);

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let mfm = Xfx::new(params, iter, map1, filter, map2);

    let (_, output) = mfm.reduce::<DefaultRunner, _>(reduce);

    assert_eq!(expected, output);
}
