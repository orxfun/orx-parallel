use crate::{Params, runner::ParRunner};

pub struct RunnerWithDiagnostics<R: ParRunner>(R);

impl<R: ParRunner> ParRunner for RunnerWithDiagnostics<R> {
    type Pool = R::Pool;

    type State = R::State;

    type ChunkState = R::ChunkState;

    fn pool(&self) -> &Self::Pool {
        todo!()
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        todo!()
    }

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
        todo!()
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        initial_len: Option<usize>,
    ) -> Self::State {
        todo!()
    }

    fn next_chunk_size(state: &Self::State, remaining: Option<usize>) -> usize {
        todo!()
    }

    fn begin_chunk(th_idx: usize, chunk_size: usize) -> Self::ChunkState {
        todo!()
    }

    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState) {
        todo!()
    }

    fn complete_computation(state: Self::State) {
        todo!()
    }
}
