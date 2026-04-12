use crate::{Params, runner::ParRunner};
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;

pub struct RunnerWithDiagnostics<R: ParRunner>(R);

impl<R: ParRunner> ParRunner for RunnerWithDiagnostics<R> {
    type Pool = R::Pool;

    type State = StateWithDiagnostics<R::State>;

    type ChunkState = R::ChunkState;

    fn pool(&self) -> &Self::Pool {
        self.0.pool()
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        self.0.pool_mut()
    }

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
        R::do_spawn_new(spawned, &state.inner)
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        initial_len: Option<usize>,
    ) -> Self::State {
        let inner = self.0.new_state(params, max_num_threads, initial_len);
        StateWithDiagnostics::new(inner)
    }

    fn next_chunk_size(state: &Self::State, remaining: Option<usize>) -> usize {
        R::next_chunk_size(&state.inner, remaining)
    }

    fn begin_chunk(th_idx: usize, chunk_size: usize) -> Self::ChunkState {
        R::begin_chunk(th_idx, chunk_size)
    }

    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState) {
        R::complete_chunk(&state.inner, chunk_state);
    }

    fn complete_computation(state: Self::State) {
        R::complete_computation(state.inner);
    }
}

pub struct StateWithDiagnostics<S> {
    inner: S,
    task_counts: ConcurrentBag<(usize, Vec<usize>)>, // (thread_idx, chunk sizes)
}

impl<S> StateWithDiagnostics<S> {
    pub fn new(inner: S) -> Self {
        let task_counts = ConcurrentBag::new();
        Self { inner, task_counts }
    }
}
