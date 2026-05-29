use crate::infallible_use::XapUse;
use crate::results::ValIdx;
use crate::runner::ParRunner;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn next<Q, U, I, X>(
    u: &mut U,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x: X,
) -> Option<ValIdx<X::O>>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: XapUse<U = U, I = I::Item>,
{
    let mut chunk_puller = iter.chunk_puller_by(0, th_idx);

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match iter.next_with_idx_by(th_idx) {
                Some((idx, i)) => {
                    if let Some(val) = x.xap_use(u, i).into_iter().next() {
                        Q::broadcast_stop(iter, state, chunk_state);
                        return Some(ValIdx::new(val, idx));
                    }
                }
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                chunk_puller.resize_for_chunk_size(c);

                match chunk_puller.pull_with_idx() {
                    Some((idx, chunk)) => {
                        if let Some(val) = chunk.flat_map(|i| x.xap_use(u, i)).next() {
                            Q::broadcast_stop(iter, state, chunk_state);
                            return Some(ValIdx::new(val, idx));
                        }
                    }
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }

        Q::complete_chunk(state, chunk_state);
    }

    None
}
