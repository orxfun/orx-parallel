use crate::runner::par_runner::ParRunner;
use crate::runner::runner_variants::fixed_chunk::{heuristic, state::State};
use crate::{parameters::Params, pool::ParThreadPool};

pub struct FixedChunkRunner<P: ParThreadPool> {
    pool: P,
}

unsafe impl<P: ParThreadPool> Sync for FixedChunkRunner<P> {}

impl<P: ParThreadPool> FixedChunkRunner<P> {
    pub fn new(pool: P) -> Self {
        Self { pool }
    }
}

impl<P: ParThreadPool> ParRunner for FixedChunkRunner<P> {
    type Pool = P;

    type State = State;

    type ChunkState = ();

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.pool
    }

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
        (spawned < state.max_num_threads).then_some(spawned)
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        initial_len: Option<usize>,
    ) -> Self::State {
        let chunk_size =
            heuristic::compute_chunk_size(params.chunk_size, initial_len, max_num_threads);
        State {
            max_num_threads,
            initial_len,
            chunk_size,
        }
    }

    #[inline(always)]
    fn begin_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn next_chunk_size(state: &Self::State, _: Option<usize>) -> usize {
        state.chunk_size
    }

    #[inline(always)]
    fn begin_chunk(_: usize, _: usize) -> Self::ChunkState {}

    #[inline(always)]
    fn complete_chunk(_: &Self::State, _: Self::ChunkState) {}

    #[inline(always)]
    fn complete_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn complete_computation(_: Self::State) {}
}
