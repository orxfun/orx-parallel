use super::run_map;
use crate::{
    IterationOrder, RunnerWithPool, StdRunner, runner::implementations::std_runner::StdDefaultPool,
};
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
fn pool_scoped_threadpool_map(n: usize, _: usize, chunk: usize, ordering: IterationOrder) {
    let orch = StdRunner::default();
    run_map(n, chunk, ordering, orch);

    let pool = StdDefaultPool::default();
    let orch: RunnerWithPool<_> = (&pool).into();
    run_map(n, chunk, ordering, orch);
}
