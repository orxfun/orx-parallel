use crate::computations::{Atom, Values};
use crate::using::UsingClone;
use crate::using::computations::UXfx;
use crate::{IterationOrder, Params, runner::DefaultRunner};
use orx_concurrent_iter::IntoConcurrentIter;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64],
    [IterationOrder::Ordered, IterationOrder::Arbitrary],
    [true, false])
]
fn u_xfx_map_filter_collect(
    n: usize,
    nt: usize,
    chunk: usize,
    ordering: IterationOrder,
    actual_filter: bool,
) {
    let offset = 33;

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
        Atom(x)
    };

    let mut output = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
    let mut expected = Vec::new();

    let mut u = 0;
    let mut u2 = 0;
    for i in 0..offset {
        let value = map1(&mut u, i.to_string()).first().unwrap();
        if filter(&mut u, &value) {
            output.push(value.clone());
            expected.push(value);
        }
    }
    expected.extend(
        input
            .clone()
            .into_iter()
            .flat_map(|x| map1(&mut u, x).values())
            .filter(|x| filter(&mut u2, x)),
    );

    let params = Params::new(nt, chunk, ordering);
    let iter = input.into_con_iter();
    let mfm = UXfx::new(UsingClone::new(0), params, iter, map1, filter, map2);

    let (_, mut output) = mfm.collect_into::<DefaultRunner, _>(output);

    if !params.is_sequential() && matches!(params.iteration_order, IterationOrder::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64],
    [IterationOrder::Ordered, IterationOrder::Arbitrary],
    [true, false])
]
fn u_xfx_filter_collect(
    n: usize,
    nt: usize,
    chunk: usize,
    ordering: IterationOrder,
    actual_filter: bool,
) {
    let offset = 33;

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map = |u: &mut usize, x: String| {
        *u += 1;
        Atom(x)
    };
    let filter = move |u: &mut usize, x: &String| {
        *u += 1;
        match actual_filter {
            true => !x.starts_with('1'),
            false => true,
        }
    };

    let mut output = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
    let mut expected = Vec::new();

    let mut u = 0;
    for i in 0..offset {
        let value = i.to_string();
        if filter(&mut u, &value) {
            output.push(value.clone());
            expected.push(value);
        }
    }
    expected.extend(input.clone().into_iter().filter(|x| filter(&mut u, x)));

    let params = Params::new(nt, chunk, ordering);
    let iter = input.into_con_iter();
    let mfm = UXfx::new(UsingClone::new(0), params, iter, map, filter, map);

    let (_, mut output) = mfm.collect_into::<DefaultRunner, _>(output);

    if !params.is_sequential() && matches!(params.iteration_order, IterationOrder::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64],
    [IterationOrder::Ordered, IterationOrder::Arbitrary],
    [true, false])
]
fn u_xfx_map_filter_map_collect(
    n: usize,
    nt: usize,
    chunk: usize,
    ordering: IterationOrder,
    actual_filter: bool,
) {
    let offset = 33;

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

    let mut output = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
    let mut expected = Vec::new();

    let mut u = 0;
    let mut u2 = 0;
    let mut u3 = 0;
    for i in 0..offset {
        let value = map1(&mut u, i.to_string()).first().unwrap();
        if filter(&mut u, &value) {
            let value = map2(&mut u, value).first().unwrap();
            output.push(value.clone());
            expected.push(value);
        }
    }
    expected.extend(
        input
            .clone()
            .into_iter()
            .flat_map(|x| map1(&mut u, x).values())
            .filter(|x| filter(&mut u2, x))
            .flat_map(|x| map2(&mut u3, x).values()),
    );

    let params = Params::new(nt, chunk, ordering);
    let iter = input.into_con_iter();
    let mfm = UXfx::new(UsingClone::new(0), params, iter, map1, filter, map2);

    let (_, mut output) = mfm.collect_into::<DefaultRunner, _>(output);

    if !params.is_sequential() && matches!(params.iteration_order, IterationOrder::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}
