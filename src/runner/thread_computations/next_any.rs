use crate::runner::{ParRunner, thread_computations::sync::broadcast_stop};
use crate::xap::Xap;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn next_any<Q, I, X>(th_idx: usize, state: &Q::State, iter: &I, x: X) -> Option<X::O>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    if let Some(val) = x.xap(i).into_iter().next() {
                        broadcast_stop::<_, Q>(iter, state, chunk_state);
                        return Some(val);
                    }
                }
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => {
                        if let Some(val) = chunk.flat_map(|i| x.xap(i).into_iter()).next() {
                            broadcast_stop::<_, Q>(iter, state, chunk_state);
                            return Some(val);
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

pub fn option_next_any<Q, I, X, O>(th_idx: usize, state: &Q::State, iter: &I, x: X) -> Option<O>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Option<O>>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    if let Some(val) = x.xap(i).into_iter().next() {
                        broadcast_stop::<_, Q>(iter, state, chunk_state);
                        // return Some(val);
                        todo!()
                    }
                }
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => {
                        if let Some(val) = chunk.flat_map(|i| x.xap(i).into_iter()).next() {
                            broadcast_stop::<_, Q>(iter, state, chunk_state);
                            // return Some(val);
                            todo!()
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
