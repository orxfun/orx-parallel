use crate::infallible_use::{XapUse, use_var::Use};
use crate::runner::ParRunner;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn next_any<Q, U, I, X>(u: &U, th_idx: usize, state: &Q::State, iter: &I, x: X) -> Option<X::O>
where
    Q: ParRunner,
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item>,
{
    let mut u = u.create(th_idx);
    let u = &mut u as *mut U::Item;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);
        let mut non_empty = false;

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    non_empty = true;
                    if let Some(val) = x.xap_use(u, i).into_iter().next() {
                        Q::broadcast_stop(iter, state, chunk_state);
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
                        non_empty = true;
                        if let Some(val) = chunk.flat_map(|i| x.xap_use(u, i)).next() {
                            Q::broadcast_stop(iter, state, chunk_state);
                            return Some(val);
                        }
                    }
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }
        match non_empty {
            true => Q::complete_chunk_non_empty(state, chunk_state),
            false => Q::complete_chunk_empty(state, chunk_state),
        }
    }

    None
}
