use crate::option::size_pairs::SizePairOpt;
use crate::{infallible::Xap, results::ValIdx, runner::ParRunner};
use alloc::vec::Vec;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn collect<Q, I, M, X1, X2, S>(
    _: S,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x1: X1,
    x2: X2,
) -> Option<Vec<ValIdx<X2::O>>>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M>,
    S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
{
    let mut collected = Vec::new();
    let vec = &mut collected;

    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    for a in S::xap_opt(x1, x2, i) {
                        match a {
                            Some(a) => vec.push(ValIdx::new(a, idx)),
                            None => {
                                Q::broadcast_stop(iter, state, chunk_state);
                                return None;
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

                match chunk_puller.pull_with_idx() {
                    Some((idx, chunk)) => {
                        for a in chunk.flat_map(|i| S::xap_opt(x1, x2, i)) {
                            match a {
                                Some(a) => vec.push(ValIdx::new(a, idx)),
                                None => {
                                    Q::broadcast_stop(iter, state, chunk_state);
                                    return None;
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

    Some(collected)
}
