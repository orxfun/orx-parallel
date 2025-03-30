use crate::computations::{map_self_atom, Atom, Values};
use crate::{
    computations::xap_filter_xap::xfx::Xfx, runner::DefaultRunner, CollectOrdering, Params,
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
    [1, 2, 4],
    [1, 64, 1024],
    [CollectOrdering::SortWithHeap, CollectOrdering::Arbitrary],
    [true, false])
]
fn xfx_map_filter(
    n: usize,
    nt: usize,
    chunk: usize,
    ordering: CollectOrdering,
    actual_filter: bool,
) {
    let offset = 33;

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map1 = |x: String| Atom(format!("{}!", x));
    let filter = move |x: &String| match actual_filter {
        true => !x.starts_with('1'),
        false => true,
    };

    let mut output = SplitVec::with_doubling_growth_and_fragments_capacity(32);
    let mut expected = Vec::new();

    for i in 0..offset {
        let value = || map1(i.to_string()).first().unwrap();
        if filter(&value()) {
            output.push(value());
            expected.push(value());
        }
    }
    expected.extend(
        input
            .clone()
            .into_iter()
            .flat_map(|x| map1(x).values())
            .filter(&filter),
    );

    let params = Params::new(nt, chunk, ordering);
    let iter = input.into_con_iter();
    let mfm = Xfx::new(params, iter, map1, filter, map_self_atom);

    let (_, mut output) = mfm.collect_into::<DefaultRunner, _>(output);

    if !params.is_sequential() && matches!(params.collect_ordering, CollectOrdering::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [CollectOrdering::SortWithHeap, CollectOrdering::Arbitrary],
    [true, false])
]
fn xfx_filter(n: usize, nt: usize, chunk: usize, ordering: CollectOrdering, actual_filter: bool) {
    let offset = 33;

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let filter = move |x: &String| match actual_filter {
        true => !x.starts_with('1'),
        false => true,
    };

    let mut output = SplitVec::with_doubling_growth_and_fragments_capacity(32);
    let mut expected = Vec::new();

    for i in 0..offset {
        let value = || i.to_string();
        if filter(&value()) {
            output.push(value());
            expected.push(value());
        }
    }
    expected.extend(input.clone().into_iter().filter(&filter));

    let params = Params::new(nt, chunk, ordering);
    let iter = input.into_con_iter();
    let mfm = Xfx::new(params, iter, map_self_atom, filter, map_self_atom);

    let (_, mut output) = mfm.collect_into::<DefaultRunner, _>(output);

    if !params.is_sequential() && matches!(params.collect_ordering, CollectOrdering::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024],
    [CollectOrdering::SortWithHeap, CollectOrdering::Arbitrary],
    [true, false])
]
fn xfx_map_filter_map(
    n: usize,
    nt: usize,
    chunk: usize,
    ordering: CollectOrdering,
    actual_filter: bool,
) {
    let offset = 33;

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map1 = |x: String| Atom(format!("{}!", x));
    let filter = move |x: &String| match actual_filter {
        true => !x.starts_with('1'),
        false => true,
    };
    let map2 = |x: String| Atom(x.len());

    let mut output = SplitVec::with_doubling_growth_and_fragments_capacity(32);
    let mut expected = Vec::new();

    for i in 0..offset {
        let value = map1(i.to_string()).first().unwrap();
        if filter(&value) {
            let value = map2(value).first().unwrap();
            output.push(value.clone());
            expected.push(value);
        }
    }
    expected.extend(
        input
            .clone()
            .into_iter()
            .flat_map(|x| map1(x).values())
            .filter(&filter)
            .flat_map(|x| map2(x).values()),
    );

    let params = Params::new(nt, chunk, ordering);
    let iter = input.into_con_iter();
    let mfm = Xfx::new(params, iter, map1, filter, map2);

    let (_, mut output) = mfm.collect_into::<DefaultRunner, _>(output);

    if !params.is_sequential() && matches!(params.collect_ordering, CollectOrdering::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}
