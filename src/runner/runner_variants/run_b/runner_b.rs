use crate::parameters::{ChunkSize, Params};
use crate::pool::ParThreadPool;
use crate::runner::par_runner::ParRunner;
use crate::runner::runner_variants::run_b::state::{ChunkState, Mode, State};
use core::cmp::min;

pub struct RunnerB<P: ParThreadPool> {
    pool: P,
}

unsafe impl<P: ParThreadPool> Sync for RunnerB<P> {}

impl<P: ParThreadPool> RunnerB<P> {
    pub fn new(pool: P) -> Self {
        Self { pool }
    }
}

impl<P: ParThreadPool> ParRunner for RunnerB<P> {
    type Pool = P;

    type State = State;

    type ChunkState = crate::runner::runner_variants::run_b::state::ChunkState;

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
        RunnerB::new(pool)
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
        let min_chunk_size = match params.chunk_size {
            ChunkSize::Auto => 1,
            ChunkSize::Min(chunk_size) | ChunkSize::Exact(chunk_size) => chunk_size.into(),
        };

        let fixed_chunk_size = match params.chunk_size {
            ChunkSize::Exact(chunk_size) => Some(chunk_size.into()),
            _ => None,
        };

        State {
            max_num_threads,
            min_chunk_size,
            fixed_chunk_size,
            initial_len: match _size_hint.1 {
                Some(upper_bound) if upper_bound == _size_hint.0 => Some(upper_bound),
                _ => None,
            },
            explore_started_at: std::time::Instant::now(),
            mode: core::sync::atomic::AtomicUsize::new(0),
            chosen_chunk_size: core::sync::atomic::AtomicUsize::new(fixed_chunk_size.unwrap_or(0)),
            explored_tasks: core::sync::atomic::AtomicUsize::new(0),
            avg_ns_per_item: core::sync::atomic::AtomicU64::new(0),
            avg_abs_deviation_ns_per_item: core::sync::atomic::AtomicU64::new(0),
            prev_avg_ns_per_item: core::sync::atomic::AtomicU64::new(0),
            converged_samples: core::sync::atomic::AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    fn begin_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn next_chunk_size(state: &Self::State, size_hint: (usize, Option<usize>)) -> usize {
        if let Some(fixed_chunk_size) = state.fixed_chunk_size {
            let remaining = size_hint.1.unwrap_or(size_hint.0).max(1);
            return min(fixed_chunk_size, remaining);
        }

        match state.mode() {
            Mode::Explore => min(
                state.min_chunk_size,
                size_hint.1.unwrap_or(size_hint.0).max(1),
            ),
            Mode::Fixed => state.selected_chunk_size(size_hint),
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
