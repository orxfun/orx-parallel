use crate::{pool::ParThreadPool, runner::par_runner::ParRunner};

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

    type State = ();

    type ChunkState = ();

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.pool
    }

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
        todo!()
    }

    fn next_chunk_size(state: &Self::State, remaining: Option<usize>) -> usize {
        todo!()
    }

    fn new_state(&mut self) -> Self::State {
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
