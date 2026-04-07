use crate::result::{size_pairs::SizePair, xap_res::XapRes};
use crate::{infallible::Xap, results::ValIdx, runner::ParRunner};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn next<Q, I, M, E, X1, X2, S>(
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x: XapRes<M, E, X1, X2, S>,
) -> Result<Option<ValIdx<X2::O>>, E>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    for a in x.xap_res(i) {
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
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((idx, chunk)) => {
                        for a in chunk.flat_map(|i| x.xap_res(i)) {
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
