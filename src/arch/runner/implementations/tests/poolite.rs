use super::run_map;
use crate::{IterationOrder, runner::implementations::RunnerWithPool};
use poolite::{Builder, Pool};
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

// `poolite::Builder::new()` fails miri test
#[cfg(not(miri))]
#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64],
    [IterationOrder::Ordered, IterationOrder::Arbitrary])
]
fn pool_poolite_map(n: usize, nt: usize, chunk: usize, ordering: IterationOrder) {
    let pool = Pool::with_builder(Builder::new().max(nt).min(nt)).unwrap();
    let orch: RunnerWithPool<_> = (&pool).into();
    run_map(n, chunk, ordering, orch);
}
