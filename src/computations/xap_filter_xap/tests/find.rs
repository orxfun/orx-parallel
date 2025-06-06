use crate::computations::{Atom, Values, map_self_atom};
use crate::{Params, computations::xap_filter_xap::xfx::Xfx, runner::DefaultRunner};
use orx_concurrent_iter::IntoConcurrentIter;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [10285, 45735];

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn xfx_map_filter_find(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map1 = |x: String| Atom(format!("{}!", x));
    let filter = move |x: &String| match actual_filter {
        true => !x.starts_with('1'),
        false => true,
    };

    let expected = input
        .clone()
        .into_iter()
        .map(|x| map1(x).first().unwrap())
        .filter(filter)
        .next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = Xfx::new(params, iter, map1, filter, map_self_atom);

    let (_, output) = xfx.next::<DefaultRunner>();

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn xfx_filter_find(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let filter = move |x: &String| match actual_filter {
        true => !x.starts_with('1'),
        false => true,
    };

    let expected = input.clone().into_iter().filter(filter).next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = Xfx::new(params, iter, map_self_atom, filter, map_self_atom);

    let (_, output) = xfx.next::<DefaultRunner>();

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn xfx_map_filter_map_find(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map1 = |x: String| Atom(format!("{}!", x));
    let filter = move |x: &String| match actual_filter {
        true => !x.starts_with('1'),
        false => true,
    };
    let map2 = |x: String| Atom(x.len());

    let expected = input
        .clone()
        .into_iter()
        .map(|x| map1(x).first().unwrap())
        .filter(filter)
        .map(|x| map2(x).first().unwrap())
        .next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = Xfx::new(params, iter, map1, filter, map2);

    let (_, output) = xfx.next::<DefaultRunner>();

    assert_eq!(expected, output);
}

// any

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn xfx_map_filter_find_any(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map1 = |x: String| Atom(format!("{}!", x));
    let filter = move |x: &String| match actual_filter {
        true => x.contains("999"),
        false => true,
    };

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = Xfx::new(params, iter, map1, filter, map_self_atom);

    let (_, output) = xfx.next_any::<DefaultRunner>();

    assert!(output.as_ref().map(filter).unwrap_or(true));
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn xfx_filter_find_any(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let filter = move |x: &String| match actual_filter {
        true => x.contains("999"),
        false => true,
    };

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = Xfx::new(params, iter, map_self_atom, filter, map_self_atom);

    let (_, output) = xfx.next_any::<DefaultRunner>();

    assert!(output.as_ref().map(filter).unwrap_or(true));
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn xfx_map_filter_map_find_any(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map1 = |x: String| Atom(format!("{}!", x));
    let filter = move |x: &String| match actual_filter {
        true => x.contains("999"),
        false => true,
    };
    let map2 = |x: String| Atom(x);

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = Xfx::new(params, iter, map1, filter, map2);

    let (_, output) = xfx.next::<DefaultRunner>();

    assert!(output.as_ref().map(filter).unwrap_or(true));
}
