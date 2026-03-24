use crate::{pool::ParThreadPool, runner::spawned::Spawned};

pub trait Runner {
    /// Underlying thread pool.
    type Pool: ParThreadPool;

    /// Parallel computation state that is shared among thread computations.
    type State: Send + Sync;

    type ChunkState;

    // required

    /// Reference to the underlying thread pool.
    fn pool(&self) -> &Self::Pool;

    /// Mutable reference to the underlying thread pool.
    fn pool_mut(&mut self) -> &mut Self::Pool;

    // required - orchestration

    /// Returns true if it is beneficial to spawn a new thread provided that:
    ///
    /// * `spawned` threads are already been spawned, and
    /// * `state` is the current parallel execution state.
    fn do_spawn_new(&self, spawned: Spawned, shared_state: &Self::State) -> bool;

    /// Returns the next chunk size to be pulled from the input with `remaining` length
    /// for the current `shared_state`.
    fn next_chunk_size(&self, state: &Self::State, remaining: Option<usize>) -> usize;

    // required - state updates

    /// Creates an initial state for a new parallel computation.
    fn new_state(&self) -> Self::State;

    fn begin_chunk(&self, chunk_size: usize) -> Self::ChunkState;

    fn complete_chunk(&self, state: &Self::State, chunk_state: Self::ChunkState);

    fn complete_computation(&self, shared_state: Self::State);

    // provided

    fn next() {
        //
    }
}
