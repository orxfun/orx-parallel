use crate::collect_into::collect::map_collect_into;
use crate::{IterationOrder, Params, orch::DefaultOrchestrator};
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

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64],
    [IterationOrder::Ordered, IterationOrder::Arbitrary])
]
fn m_map_collect(n: usize, nt: usize, chunk: usize, ordering: IterationOrder) {
    let offset = 33;

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map = |x: String| format!("{}!", x);

    let mut output = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
    let mut expected = Vec::new();

    for i in 0..offset {
        let value = || map(i.to_string());
        output.push(value());
        expected.push(value());
    }
    expected.extend(input.clone().into_iter().map(|x| map(x)));

    let params = Params::new(nt, chunk, ordering);
    let iter = input.into_con_iter();
    let (_, mut output) =
        map_collect_into(DefaultOrchestrator::default(), params, iter, map, output);

    if !params.is_sequential() && matches!(params.iteration_order, IterationOrder::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}
