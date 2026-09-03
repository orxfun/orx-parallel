use crate::ParExtend;
use crate::sizes::SizePair;
use crate::{infallible_use::XapUse, runner::ParRunner};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn collect_x<Q, U, I, M, X1, X2, S, P>(
    _: S,
    u: &mut U,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x1: X1,
    x2: X2,
) -> Option<P::OrderedThreadValues>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X1: XapUse<U = U, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U, I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    X2::O: Send,
    P: ParExtend<X2::O>,
{
    let mut collected = P::new_ordered_thread_values();
    let out = &mut collected;
    let u = u as *mut U;

    let mut chunk_puller = iter.chunk_puller_by(0, th_idx);

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match iter.next_with_idx_by(th_idx) {
                Some((idx, i)) => {
                    let values = S::xap_use_opt(u, x1, x2, i);
                    let result = P::add_ordered_thread_optionals(out, idx, values);
                    if result.is_none() {
                        Q::broadcast_stop(iter, state, chunk_state);
                        return None;
                    }
                }
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                chunk_puller.resize_for_chunk_size(c);

                match chunk_puller.pull_with_idx() {
                    Some((idx, chunk)) => {
                        let values = chunk.flat_map(|i| S::xap_use_opt(u, x1, x2, i));
                        let result = P::add_ordered_thread_optionals(out, idx, values);
                        if result.is_none() {
                            Q::broadcast_stop(iter, state, chunk_state);
                            return None;
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
