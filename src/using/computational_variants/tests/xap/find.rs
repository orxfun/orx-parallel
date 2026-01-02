use crate::ParIterUsing;
use crate::Params;
use crate::generic_values::Vector;
use crate::runner::DefaultRunner;
use crate::using::UsingClone;
use crate::using::computational_variants::UParXap;
use crate::using::computational_variants::tests::utils::make_u_map;
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
fn x_flat_map_find(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let fmap = |x: String| x.chars().map(|x| x.to_string()).collect::<Vec<_>>();
    let xmap = |x: String| Vector(fmap(x));

    let expected = input.clone().into_iter().flat_map(fmap).next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let x = UParXap::new(
        UsingClone::new("XyZw".to_string()),
        DefaultRunner::default(),
        params,
        iter,
        make_u_map(xmap),
    );

    let output = x.first();

    assert_eq!(expected, output);
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64])
]
fn x_filter_map_find(n: usize, nt: usize, chunk: usize) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let fmap = |x: String| (!x.starts_with('3')).then_some(format!("{}!", x));
    let xmap = |x: String| Vector(fmap(x));

    let expected = input.clone().into_iter().filter_map(fmap).next();

    let params = Params::new(nt, chunk, Default::default());
    let iter = input.into_con_iter();
    let x = UParXap::new(
        UsingClone::new("XyZw".to_string()),
        DefaultRunner::default(),
        params,
        iter,
        make_u_map(xmap),
    );

    let output = x.first();

    assert_eq!(expected, output);
}
