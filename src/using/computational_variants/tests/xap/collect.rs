use crate::generic_values::Vector;
use crate::runner::DefaultRunner;
use crate::using::UsingClone;
use crate::using::computational_variants::UParXap;
use crate::using::computational_variants::tests::utils::make_u_map;
use crate::using::u_par_iter::ParIterUsing;
use crate::{IterationOrder, Params};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use orx_concurrent_iter::IntoConcurrentIter;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

#[test]
fn todo_panic_at_con_bag_new() {
    // TODO: this code panics when ParThreadPool::map uses ConcurrentBag::new rather than ConcurrentBag::with_fixed_capacity
    let n = 10;
    let nt = 2;
    let chunk = 1;
    let ordering = IterationOrder::Arbitrary;

    let offset = 33;

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let fmap = |x: String| x.chars().map(|x| x.to_string()).collect::<Vec<_>>();
    let xmap = |x: String| Vector(fmap(x));

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
    let x = UParXap::new(
        UsingClone::new("XyZw".to_string()),
        DefaultRunner::default(),
        params,
        iter,
        make_u_map(xmap),
    );

    let mut output = x.collect_into(output);

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
fn x_flat_map_collect(n: usize, nt: usize, chunk: usize, ordering: IterationOrder) {
    let offset = 33;

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let fmap = |x: String| x.chars().map(|x| x.to_string()).collect::<Vec<_>>();
    let xmap = |x: String| Vector(fmap(x));

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
    let x = UParXap::new(
        UsingClone::new("XyZw".to_string()),
        DefaultRunner::default(),
        params,
        iter,
        make_u_map(xmap),
    );

    let mut output = x.collect_into(output);

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
fn x_filter_map_collect(n: usize, nt: usize, chunk: usize, ordering: IterationOrder) {
    let offset = 33;

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let fmap = |x: String| (!x.starts_with('3')).then_some(format!("{}!", x));
    let xmap = |x: String| Vector(fmap(x));

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
    let x = UParXap::new(
        UsingClone::new("XyZw".to_string()),
        DefaultRunner::default(),
        params,
        iter,
        make_u_map(xmap),
    );

    let mut output = x.collect_into(output);

    if !params.is_sequential() && matches!(params.iteration_order, IterationOrder::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}
