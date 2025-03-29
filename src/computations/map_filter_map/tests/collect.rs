use crate::computations::{Atom, Values};
use crate::{
    computations::map_filter_map::mfm::Mfm, runner::DefaultRunner, CollectOrdering, Params,
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
fn mf(n: usize, nt: usize, chunk: usize, ordering: CollectOrdering, actual_filter: bool) {
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
        let value = || map1(i.to_string()).values().into_iter().next().unwrap();
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
    let mfm = Mfm::new(params, iter, map1, filter, |x| Atom(x));

    let (_, mut output) = mfm.collect_into::<DefaultRunner, _>(output);

    if !params.is_sequential() && matches!(params.collect_ordering, CollectOrdering::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}
