use crate::sizes::SizePair;
use crate::{infallible::Xap, results::ValIdx, runner::ParRunner};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn next<Q, I, M, E, X1, X2, S>(
    _: S,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x1: X1,
    x2: X2,
) -> Result<Option<ValIdx<X2::O>>, E>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    let mut chunk_puller = iter.chunk_puller_by(0, th_idx);

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match iter.next_with_idx_by(th_idx) {
                Some((idx, i)) => {
                    for a in S::xap_res(x1, x2, i) {
                        Q::broadcast_stop(iter, state, chunk_state);
                        match a {
                            Ok(a) => return Ok(Some(ValIdx::new(a, idx))),
                            Err(e) => return Err(e),
                        }
                    }
                }
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                chunk_puller.resize_for_chunk_size(c);

                match chunk_puller.pull_with_idx() {
                    Some((idx, chunk)) => {
                        for a in chunk.flat_map(|i| S::xap_res(x1, x2, i)) {
                            Q::broadcast_stop(iter, state, chunk_state);
                            match a {
                                Ok(a) => return Ok(Some(ValIdx::new(a, idx))),
                                Err(e) => return Err(e),
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

    Ok(None)
}
