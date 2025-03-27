use super::batch_of_four_runner::BatchOfFourParallelRunner;
use crate::{
    collect_into::ParCollectIntoCore,
    computations::{computation_kind::ComputationKind, runner::FixedChunkRunner, ParallelRunner},
    into_par::IntoPar,
    par_iterators::ParIter,
    parameters::Params,
};
use orx_fixed_vec::FixedVec;
use orx_iterable::Collection;
use orx_split_vec::{Doubling, Linear, PseudoDefault, SplitVec};
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

fn input<O: FromIterator<String>>(n: usize) -> O {
    let elem = |x: usize| (x + 10).to_string();
    (0..n).map(elem).collect()
}

fn expected(input: &impl Collection<Item = String>, map: impl Fn(String) -> String) -> Vec<String> {
    input.iter().cloned().map(map).collect()
}

#[test_matrix(
    [fixed_chunk_runner(), batch_of_four_runner()],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn map_collect<R: ParallelRunner>(_: R, n: usize, nt: usize, chunk: usize) {
    let map = |x: &String| format!("{}!", x);
    let map2 = |x: String| format!("{}!", x);

    let vec = input::<Vec<String>>(n);
    let input = vec.as_slice();
    let expected = expected(&vec, map2);

    let par = input
        .into_par()
        .num_threads(nt)
        .chunk_size(chunk)
        .with_runner::<R>();
    let output: Vec<String> = par.map(map).collect();
    assert!(output.is_equal_to(expected.as_slice()));
}

fn fixed_chunk_runner() -> FixedChunkRunner {
    FixedChunkRunner::new(ComputationKind::Collect, Params::default(), Some(0))
}

fn batch_of_four_runner() -> BatchOfFourParallelRunner {
    BatchOfFourParallelRunner::new(ComputationKind::Collect, Params::default(), Some(0))
}

#[test]
fn xyz() {
    let n = 100;
    let nt = 2;
    let chunk = 8;

    let map = |x: &String| format!("{}!", x);
    let map2 = |x: String| format!("{}!", x);

    let vec = input::<Vec<String>>(n);
    let input = vec.as_slice();
    let expected = expected(&vec, map2);

    let par = input.into_par().num_threads(nt).chunk_size(chunk);
    let par = par.with_runner::<BatchOfFourParallelRunner>();
    let output: Vec<String> = par.map(map).collect();

    // dbg!(&output);

    assert!(!output.is_equal_to(expected.as_slice()));
}
