use crate::{
    IterationOrder, Params,
    generic_values::Vector,
    runner::DefaultRunner,
    using::{UsingClone, computations::UX},
};
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
    [IterationOrder::Ordered, IterationOrder::Arbitrary])
]
fn u_x_flat_map_collect(n: usize, nt: usize, chunk: usize, ordering: IterationOrder) {
    let offset = 33;

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let fmap = |x: String| x.chars().map(|x| x.to_string()).collect::<Vec<_>>();
    let xmap = |u: &mut usize, x: String| {
        *u += 1;
        Vector(fmap(x))
    };

    let mut output = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
    let mut expected = Vec::new();

    for i in 0..offset {
        let i = i.to_string();
        for x in fmap(i) {
            output.push(x.clone());
            expected.push(x);
        }
    }
    expected.extend(input.clone().into_iter().flat_map(&fmap));

    let params = Params::new(nt, chunk, ordering);
    let iter = input.into_con_iter();
    let x = UX::new(UsingClone::new(0), params, iter, xmap);

    let (_, mut output) = x.collect_into::<DefaultRunner, _>(output);

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
    [IterationOrder::Ordered, IterationOrder::Arbitrary])
]
fn u_x_filter_map_collect(n: usize, nt: usize, chunk: usize, ordering: IterationOrder) {
    let offset = 33;

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let fmap = |x: String| (!x.starts_with('3')).then_some(format!("{}!", x));
    let xmap = |u: &mut usize, x: String| {
        *u += 1;
        Vector(fmap(x))
    };

    let mut output = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
    let mut expected = Vec::new();

    for i in 0..offset {
        let i = i.to_string();
        if let Some(x) = fmap(i) {
            output.push(x.clone());
            expected.push(x);
        }
    }
    expected.extend(input.clone().into_iter().flat_map(&fmap));

    let params = Params::new(nt, chunk, ordering);
    let iter = input.into_con_iter();
    let x = UX::new(UsingClone::new(0), params, iter, xmap);

    let (_, mut output) = x.collect_into::<DefaultRunner, _>(output);

    if !params.is_sequential() && matches!(params.iteration_order, IterationOrder::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}
