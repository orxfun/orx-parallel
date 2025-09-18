use super::run_map;
use crate::{IterationOrder, runner::implementations::RunnerWithRayonPool};
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

// TODO: rayon pool fails the miri test (integer-to-pointer cast crossbeam-epoch-0.9.18/src/atomic.rs:204:11)
#[cfg(not(miri))]
#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64],
    [IterationOrder::Ordered, IterationOrder::Arbitrary])
]
fn pool_rayon_map(n: usize, nt: usize, chunk: usize, ordering: IterationOrder) {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(nt)
        .build()
        .unwrap();
    let orch: RunnerWithRayonPool<_> = (&pool).into();
    run_map(n, chunk, ordering, orch);
}
