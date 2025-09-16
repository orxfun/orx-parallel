use crate::{
    DefaultRunner, Params,
    generic_values::Vector,
    using_old::{UsingClone, computations::UX},
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
fn u_x_flat_map_find(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let fmap = |x: String| x.chars().map(|x| x.to_string()).collect::<Vec<_>>();
    let xmap = |u: &mut usize, x: String| {
        *u += 1;
        Vector(fmap(x))
    };

    let expected = input.clone().into_iter().flat_map(fmap).next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let x = UX::new(UsingClone::new(0), params, iter, xmap);

    let output = x.next::<DefaultRunner>().1;
    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64])
]
fn u_x_filter_map_find(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let fmap = |x: String| (!x.starts_with('3')).then_some(format!("{}!", x));
    let xmap = |u: &mut usize, x: String| {
        *u += 1;
        Vector(fmap(x))
    };

    let expected = input.clone().into_iter().filter_map(fmap).next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let x = UX::new(UsingClone::new(0), params, iter, xmap);

    let output = x.next::<DefaultRunner>().1;

    assert_eq!(expected, output);
}
