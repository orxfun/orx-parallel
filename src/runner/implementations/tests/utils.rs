use crate::{IntoParIter, IterationOrder, ParIter, runner::ParallelRunner};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;

pub fn run_map(n: usize, chunk: usize, ordering: IterationOrder, mut orch: impl ParallelRunner) {
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

    let mut output = input
        .into_par()
        .with_runner(&mut orch)
        .chunk_size(chunk)
        .iteration_order(ordering)
        .map(map)
        .collect_into(output);

    if matches!(ordering, IterationOrder::Arbitrary) {
        expected.sort();
        output.sort();
    }

    assert_eq!(expected, output.to_vec());
}
