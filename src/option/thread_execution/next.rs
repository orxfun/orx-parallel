use crate::{infallible::Xap, results::ValIdx, runner::ParRunner, sizes::SizePair};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn next<Q, I, M, X1, X2, S>(
    _: S,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x1: X1,
    x2: X2,
) -> Option<Option<ValIdx<X2::O>>>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);
        let mut non_empty = false;

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    non_empty = true;
                    for a in S::xap_opt(x1, x2, i) {
                        Q::broadcast_stop(iter, state, chunk_state);
                        match a {
                            Some(a) => return Some(Some(ValIdx::new(a, idx))),
                            None => return None,
                        }
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
                        non_empty = true;
                        for a in chunk.flat_map(|i| S::xap_opt(x1, x2, i)) {
                            Q::broadcast_stop(iter, state, chunk_state);
                            match a {
                                Some(a) => return Some(Some(ValIdx::new(a, idx))),
                                None => return None,
                            }
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

    Some(None)
}
