use super::run_map;
use crate::{IterationOrder, runner::implementations::RunnerWithPool};
use scoped_pool::Pool;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

// `scoped_pool::Pool::new(nt)` fails miri test
#[cfg(not(miri))]
#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64],
    [IterationOrder::Ordered, IterationOrder::Arbitrary])
]
fn pool_scoped_pool_map(n: usize, nt: usize, chunk: usize, ordering: IterationOrder) {
    let pool = Pool::new(nt);
    let orch = RunnerWithPool::from(&pool);
    run_map(n, chunk, ordering, orch);
}
