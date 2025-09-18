use super::run_map;
use crate::{
    IterationOrder,
    runner::implementations::{RunnerWithYastlPool, YastlPool},
};
use test_case::test_matrix;
use yastl::ThreadConfig;

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
fn pool_yastl_map(n: usize, nt: usize, chunk: usize, ordering: IterationOrder) {
    let pool = YastlPool::new(nt);
    let orch: RunnerWithYastlPool<_> = (&pool).into();
    run_map(n, chunk, ordering, orch);

    let pool = YastlPool::with_config(nt, ThreadConfig::new());
    let orch: RunnerWithYastlPool<_> = (&pool).into();
    run_map(n, chunk, ordering, orch);
}
