use crate::runner::par_runner::ParRunner;
use crate::runner::runner_variants::run_b::state::State;
use crate::{parameters::Params, pool::ParThreadPool};

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

    type ChunkState = ();

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
        _: Params,
        max_num_threads: usize,
        _: (usize, Option<usize>),
    ) -> Self::State {
        State { max_num_threads }
    }

    #[inline(always)]
    fn begin_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn next_chunk_size(_: &Self::State, _: (usize, Option<usize>)) -> usize {
        1
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
