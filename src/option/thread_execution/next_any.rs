use crate::{infallible::Xap, runner::ParRunner, sizes::SizePair};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn next_any<Q, I, M, X1, X2, S>(
    _: S,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x1: X1,
    x2: X2,
) -> Option<Option<X2::O>>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    let mut chunk_puller = iter.chunk_puller_by(0, th_idx);

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match iter.next_by(th_idx) {
                Some(i) => {
                    if let Some(a) = S::xap_opt(x1, x2, i).into_iter().next() {
                        Q::broadcast_stop(iter, state, chunk_state);
                        match a {
                            Some(a) => return Some(Some(a)),
                            None => return None,
                        }
                    }
                }
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                chunk_puller.resize_for_chunk_size(c);

                match chunk_puller.pull() {
                    Some(chunk) => {
                        if let Some(a) = chunk.flat_map(|i| S::xap_opt(x1, x2, i)).next() {
                            Q::broadcast_stop(iter, state, chunk_state);
                            match a {
                                Some(a) => return Some(Some(a)),
                                None => return None,
                            }
                        }
                    }
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }
        Q::complete_chunk(state, chunk_state);
    }

    Some(None)
}
