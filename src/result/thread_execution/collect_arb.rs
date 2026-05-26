use crate::infallible::Xap;
use crate::runner::ParRunner;
use crate::sizes::SizePair;
use alloc::vec::Vec;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn collect_arb<Q, I, M, E, X1, X2, S>(
    _: S,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x1: X1,
    x2: X2,
) -> Result<Vec<X2::O>, E>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    let mut collected = Vec::new();
    let vec = &mut collected;

    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    for a in S::xap_res(x1, x2, i) {
                        match a {
                            Ok(a) => vec.push(a),
                            Err(e) => {
                                Q::broadcast_stop(iter, state, chunk_state);
                                return Err(e);
                            }
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

                match chunk_puller.pull() {
                    Some(chunk) => {
                        for a in chunk.flat_map(|i| S::xap_res(x1, x2, i)) {
                            match a {
                                Ok(a) => vec.push(a),
                                Err(e) => {
                                    Q::broadcast_stop(iter, state, chunk_state);
                                    return Err(e);
                                }
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

    Ok(collected)
}
