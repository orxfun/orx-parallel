use crate::infallible::xap::Xap;
use crate::results::ValIdx;
use crate::runner::ParRunner;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn next<Q, I, X>(th_idx: usize, state: &Q::State, iter: &I, x: X) -> Option<ValIdx<X::O>>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    if let Some(val) = x.xap(i).into_iter().next() {
                        Q::broadcast_stop(iter, state, chunk_state);
                        return Some(ValIdx::new(val, idx));
                    }
                }
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((idx, chunk)) => {
                        if let Some(val) = chunk.flat_map(|i| x.xap(i)).next() {
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
