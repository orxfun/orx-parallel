use crate::computations::{Atom, Values};
use crate::using::UsingClone;
use crate::using::computations::UXfx;
use crate::{Params, runner::DefaultRunner};
use orx_concurrent_iter::IntoConcurrentIter;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [10285, 45735];

fn map1(u: &mut usize, x: String) -> Atom<String> {
    *u += 1;
    Atom(format!("{}!", x))
}

fn map2(u: &mut usize, x: String) -> Atom<usize> {
    *u += 1;
    Atom(x.len())
}

fn map_self_atom(u: &mut usize, x: String) -> Atom<String> {
    *u += 1;
    Atom(x)
}

fn get_filter(actual_filter: bool) -> impl Fn(&mut usize, &String) -> bool {
    move |u: &mut usize, x: &String| {
        *u += 1;
        match actual_filter {
            true => !x.starts_with('1'),
            false => true,
        }
    }
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn u_xfx_map_filter_find(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();

    let filter = get_filter(actual_filter);

    let mut u = 0;
    let mut u2 = 0;
    let expected = input
        .clone()
        .into_iter()
        .map(|x| map1(&mut u, x).first().unwrap())
        .filter(|x| filter(&mut u2, x))
        .next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = UXfx::new(
        UsingClone::new(0),
        params,
        iter,
        map1,
        filter,
        map_self_atom,
    );

    let (_, output) = xfx.next::<DefaultRunner>();

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn u_xfx_filter_find(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();

    let filter = get_filter(actual_filter);

    let mut u = 0;
    let expected = input
        .clone()
        .into_iter()
        .filter(|x| filter(&mut u, x))
        .next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = UXfx::new(
        UsingClone::new(0),
        params,
        iter,
        map_self_atom,
        filter,
        map_self_atom,
    );

    let (_, output) = xfx.next::<DefaultRunner>();

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn u_xfx_map_filter_map_find(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let filter = get_filter(actual_filter);

    let mut u = 0;
    let mut u2 = 0;
    let mut u3 = 0;
    let expected = input
        .clone()
        .into_iter()
        .map(|x| map1(&mut u, x).first().unwrap())
        .filter(|x| filter(&mut u2, x))
        .map(|x| map2(&mut u3, x).first().unwrap())
        .next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = UXfx::new(UsingClone::new(0), params, iter, map1, filter, map2);

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
fn u_xfx_map_filter_find_any(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let filter = move |u: &mut usize, x: &String| {
        *u += 1;
        match actual_filter {
            true => x.contains("999"),
            false => true,
        }
    };

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = UXfx::new(
        UsingClone::new(0),
        params,
        iter,
        map1,
        filter,
        map_self_atom,
    );

    let (_, output) = xfx.next_any::<DefaultRunner>();

    let mut u = 0;
    assert!(output.as_ref().map(|x| filter(&mut u, x)).unwrap_or(true));
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn u_xfx_filter_find_any(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let filter = move |u: &mut usize, x: &String| {
        *u += 1;
        match actual_filter {
            true => x.contains("999"),
            false => true,
        }
    };

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = UXfx::new(
        UsingClone::new(0),
        params,
        iter,
        map_self_atom,
        filter,
        map_self_atom,
    );

    let (_, output) = xfx.next_any::<DefaultRunner>();

    let mut u = 0;
    assert!(output.as_ref().map(|x| filter(&mut u, x)).unwrap_or(true));
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [true, false])
]
fn u_xfx_map_filter_map_find_any(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let filter = move |u: &mut usize, x: &String| {
        *u += 1;
        match actual_filter {
            true => x.contains("999"),
            false => true,
        }
    };

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let xfx = UXfx::new(
        UsingClone::new(0),
        params,
        iter,
        map1,
        filter,
        map_self_atom,
    );

    let (_, output) = xfx.next::<DefaultRunner>();

    let mut u = 0;
    assert!(output.as_ref().map(|x| filter(&mut u, x)).unwrap_or(true));
}
