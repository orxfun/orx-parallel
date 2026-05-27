use crate::parameters::Params;
use crate::pool::ParThreadPool;
#[cfg(feature = "std")]
use crate::runner::runner_variants::WithDiagnostics;
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

    fn with_pool<Q: ParThreadPool>(
        self,
        pool: Q,
    ) -> impl ParRunner<State = Self::State, ChunkState = Self::ChunkState, Pool = Q>;

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
        size_hint: (usize, Option<usize>),
    ) -> Self::State;

    fn begin_thread(state: &Self::State, th_idx: usize);

    /// Returns the next chunk size to be pulled from the input with remaining length
    /// provided by the `size_hint` for the current `state`.
    fn next_chunk_size(state: &Self::State, size_hint: (usize, Option<usize>)) -> usize;

    fn begin_chunk(th_idx: usize, chunk_size: usize) -> Self::ChunkState;

    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState);

    fn complete_thread(state: &Self::State, th_idx: usize);

    fn complete_computation(state: Self::State);

    // provided

    #[cfg(feature = "std")]
    fn with_diagnostics(self) -> WithDiagnostics<Self> {
        WithDiagnostics::new(self)
    }

    // provided - helpers

    fn nt_state(
        &mut self,
        params: Params,
        size_hint: (usize, Option<usize>),
    ) -> (usize, Self::State) {
        let max_nt = self
            .pool()
            .max_num_threads_for_computation(params, size_hint);
        let state = self.new_state(params, max_nt, size_hint);
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
