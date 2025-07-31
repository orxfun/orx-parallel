use crate::{IterationOrder, Params, computations::w_map::m::WithM, runner::DefaultRunner};
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
    [IterationOrder::Ordered, IterationOrder::Arbitrary])
]
fn with_m_map_collect(n: usize, nt: usize, chunk: usize, ordering: IterationOrder) {
    let offset = 33;

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map = |counter: &mut usize, x: String| {
        *counter += 1;
        format!("{}!", x)
    };

    let mut output = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
    let mut expected = Vec::new();

    let mut counter = 0;
    for i in 0..offset {
        let value = || map(&mut counter, i.to_string());
        output.push(value());
        expected.push(value());
    }

    expected.extend(input.clone().into_iter().map(|x| map(&mut counter, x)));

    let params = Params::new(nt, chunk, ordering);
    let iter = input.into_con_iter();
    let with = 0;
    let m = WithM::new(params, iter, with, map);

    let (_, mut output) = m.collect_into::<DefaultRunner, _>(output);

    if !params.is_sequential() && matches!(params.iteration_order, IterationOrder::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}
