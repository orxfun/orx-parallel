use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;

pub fn broadcast_stop<I, Q>(iter: &I, state: &Q::State, chunk_state: Q::ChunkState)
where
    Q: ParRunner,
    I: ConcurrentIter,
{
    iter.skip_to_end();
    Q::complete_chunk(state, chunk_state);
}
