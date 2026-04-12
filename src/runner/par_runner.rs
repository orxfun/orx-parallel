use crate::parameters::Params;
use crate::pool::ParThreadPool;
#[cfg(feature = "std")]
use crate::runner::runner_variants::RunnerWithDiagnostics;
use orx_concurrent_iter::ConcurrentIter;

pub trait ParRunner: Sized + Sync {
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

    /// Returns Some of the index of the new thread if it is beneficial to spawn a new thread
    /// for the remaining computation; it returns None otherwise.
    ///
    /// Since this method is called sequentially, returned value will always be `Some(spawned)` if
    /// it returns some.
    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize>;

    /// Creates an initial state for a new parallel computation.
    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        initial_len: Option<usize>,
    ) -> Self::State;

    /// Returns the next chunk size to be pulled from the input with `remaining` length
    /// for the current `state`.
    fn next_chunk_size(state: &Self::State, remaining: Option<usize>) -> usize;

    fn begin_chunk(th_idx: usize, chunk_size: usize) -> Self::ChunkState;

    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState);

    fn complete_computation(state: Self::State);

    // provided

    #[cfg(feature = "std")]
    fn with_diagnostics(self) -> RunnerWithDiagnostics<Self> {
        RunnerWithDiagnostics::new(self)
    }

    // provided - helpers

    fn nt_state(&mut self, params: Params, len: Option<usize>) -> (usize, Self::State) {
        let max_nt = self.pool().max_num_threads_for_computation(params, len);
        let state = self.new_state(params, max_nt, len);
        (max_nt, state)
    }

    fn broadcast_stop<I: ConcurrentIter>(
        iter: &I,
        state: &Self::State,
        chunk_state: Self::ChunkState,
    ) {
        iter.skip_to_end();
        Self::complete_chunk(state, chunk_state);
    }
}
