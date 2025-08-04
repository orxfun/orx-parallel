use crate::computations::{Atom, Values};
use crate::using::UsingClone;
use crate::using::computations::{UXfx, u_map_self_atom};
use crate::{Params, runner::DefaultRunner};
use orx_concurrent_iter::IntoConcurrentIter;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64],
    [true, false])
]
fn xfx_map_filter_reduce(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map1 = |u: &mut usize, x: String| {
        *u += 1;
        Atom(format!("{}!", x))
    };
    let filter = move |u: &mut usize, x: &String| {
        *u += 1;
        match actual_filter {
            true => !x.starts_with('1'),
            false => true,
        }
    };
    let reduce = |u: &mut usize, x: String, y: String| {
        *u += 1;
        match x > y {
            true => x,
            false => y,
        }
    };

    let mut u1 = 0;
    let mut u2 = 0;
    let mut u3 = 0;
    let expected = input
        .clone()
        .into_iter()
        .map(|x| map1(&mut u1, x).first().unwrap())
        .filter(|x| filter(&mut u2, x))
        .reduce(|a, b| reduce(&mut u3, a, b));

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let mfm = UXfx::new(
        UsingClone::new(0),
        params,
        iter,
        map1,
        filter,
        u_map_self_atom,
    );

    let (_, output) = mfm.reduce::<DefaultRunner, _>(reduce);

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64],
    [true, false])
]
fn xfx_filter_reduce(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let filter = move |u: &mut usize, x: &String| {
        *u += 1;
        match actual_filter {
            true => !x.starts_with('1'),
            false => true,
        }
    };
    let reduce = |u: &mut usize, x: String, y: String| {
        *u += 1;
        match x > y {
            true => x,
            false => y,
        }
    };

    let mut u2 = 0;
    let mut u3 = 0;
    let expected = input
        .clone()
        .into_iter()
        .filter(|x| filter(&mut u2, x))
        .reduce(|a, b| reduce(&mut u3, a, b));

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let mfm = UXfx::new(
        UsingClone::new(0),
        params,
        iter,
        u_map_self_atom,
        filter,
        u_map_self_atom,
    );

    let (_, output) = mfm.reduce::<DefaultRunner, _>(reduce);

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64],
    [true, false])
]
fn xfx_map_filter_map_reduce(n: usize, nt: usize, chunk: usize, actual_filter: bool) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map1 = |u: &mut usize, x: String| {
        *u += 1;
        Atom(format!("{}!", x))
    };
    let filter = move |u: &mut usize, x: &String| {
        *u += 1;
        match actual_filter {
            true => !x.starts_with('1'),
            false => true,
        }
    };
    let map2 = |u: &mut usize, x: String| {
        *u += 1;
        Atom(x.len())
    };
    let reduce = |u: &mut usize, x: usize, y: usize| {
        *u += 1;
        x + y
    };

    let mut u1 = 0;
    let mut u2 = 0;
    let mut u3 = 0;
    let mut u4 = 0;
    let expected = input
        .clone()
        .into_iter()
        .map(|x| map1(&mut u1, x).first().unwrap())
        .filter(|x| filter(&mut u2, x))
        .map(|x| map2(&mut u3, x).first().unwrap())
        .reduce(|a, b| reduce(&mut u4, a, b));

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let mfm = UXfx::new(UsingClone::new(0), params, iter, map1, filter, map2);

    let (_, output) = mfm.reduce::<DefaultRunner, _>(reduce);

    assert_eq!(expected, output);
}
