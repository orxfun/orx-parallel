use super::run_map;
use crate::{
    IterationOrder,
    runner::implementations::{PondPool, RunnerWithPool},
};
use test_case::test_matrix;


const N: [usize; 2] = [37, 125];

const N: [usize; 2] = [1025, 4735];

// TODO: miri test terminates with: the main thread terminated without waiting for all remaining threads

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 4],
    [1, 64],
    [IterationOrder::Ordered, IterationOrder::Arbitrary])
]
fn pool_pond_map(n: usize, nt: usize, chunk: usize, ordering: IterationOrder) {
    let mut pool = PondPool::new_threads_unbounded(nt);
    let orch: RunnerWithPool<_> = (&mut pool).into();
    run_map(n, chunk, ordering, orch);
}
