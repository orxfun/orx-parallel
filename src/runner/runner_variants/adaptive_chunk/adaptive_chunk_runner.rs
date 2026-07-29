use super::chunk_state::ChunkState;
use super::mode::Mode;
use super::state::State;
use crate::parameters::{ChunkSize, Params};
use crate::pool::ParThreadPool;
use crate::runner::par_runner::ParRunner;

pub struct AdaptiveChunkRunner<P: ParThreadPool> {
    pool: P,
}

unsafe impl<P: ParThreadPool> Sync for AdaptiveChunkRunner<P> {}

impl<P: ParThreadPool> AdaptiveChunkRunner<P> {
    pub fn new(pool: P) -> Self {
        Self { pool }
    }
}

impl<P: ParThreadPool> ParRunner for AdaptiveChunkRunner<P> {
    type Pool = P;

    type State = State;

    type ChunkState = ChunkState;

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.pool
    }

    fn with_pool<Q: ParThreadPool>(
        self,
        pool: Q,
    ) -> impl ParRunner<State = Self::State, ChunkState = Self::ChunkState, Pool = Q> {
        AdaptiveChunkRunner::new(pool)
    }

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
        (spawned < state.max_num_threads).then_some(spawned)
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        _size_hint: (usize, Option<usize>),
    ) -> Self::State {
        debug_assert!(max_num_threads > 0);

        let min_chunk_size = match params.chunk_size {
            ChunkSize::Auto => 1,
            ChunkSize::Min(chunk_size) | ChunkSize::Exact(chunk_size) => chunk_size.into(),
        };

        let fixed_chunk_size = match params.chunk_size {
            ChunkSize::Exact(chunk_size) => Some(chunk_size.into()),
            _ => None,
        };

        let initial_len = match _size_hint.1 {
            Some(upper_bound) if upper_bound == _size_hint.0 => Some(upper_bound),
            _ => None,
        };

        State::new(
            max_num_threads,
            min_chunk_size,
            fixed_chunk_size,
            initial_len,
        )
    }

    #[inline(always)]
    fn begin_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn next_chunk_size(state: &Self::State, size_hint: (usize, Option<usize>)) -> usize {
        match state.fixed_chunk_size {
            Some(fixed_chunk_size) => fixed_chunk_size,
            None => match state.mode() {
                Mode::Explore => state.min_chunk_size,
                Mode::Fixed => state.selected_chunk_size(size_hint),
            },
        }
    }
    fn begin_chunk(_: usize, chunk_size: usize) -> Self::ChunkState {
        ChunkState::new(chunk_size)
    }

    #[inline(always)]
    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState) {
        if state.mode() == Mode::Explore {
            state.record_chunk(chunk_state);
            if state.should_stop_exploration() {
                state.complete_exploration();
            }
        }
    }

    #[inline(always)]
    fn complete_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn complete_computation(_state: Self::State) {}
}
