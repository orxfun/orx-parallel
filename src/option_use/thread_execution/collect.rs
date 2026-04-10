use crate::infallible_use::Use;
use crate::option_use::SizePairUseOpt;
use crate::{infallible_use::XapUse, results::ValIdx, runner::ParRunner};
use alloc::vec::Vec;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn collect<Q, U, I, M, X1, X2, S>(
    _: S,
    u: &U,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x1: X1,
    x2: X2,
) -> Option<Vec<ValIdx<X2::O>>>
where
    Q: ParRunner,
    U: Use,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePairUseOpt<S1 = X1::Size, S2 = X2::Size>,
{
    let mut collected = Vec::new();
    let vec = &mut collected;

    let mut u = u.create(th_idx);
    let u = &mut u as *mut U::Item;

    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    for a in S::xap_use_opt(u, x1, x2, i) {
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
                        for a in chunk.flat_map(|i| S::xap_use_opt(u, x1, x2, i)) {
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
