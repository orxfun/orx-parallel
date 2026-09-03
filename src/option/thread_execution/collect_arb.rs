use crate::ParExtend;
use crate::runner::ParRunner;
use crate::{infallible::Xap, sizes::SizePair};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn collect_arb_x<Q, I, M, X1, X2, S, P>(
    _: S,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x1: X1,
    x2: X2,
) -> Option<P::ThreadValues>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    P: ParExtend<X2::O>,
{
    let mut collected = P::new_thread_values();
    let out = &mut collected;

    let mut chunk_puller = iter.chunk_puller_by(0, th_idx);

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match iter.next_by(th_idx) {
                Some(i) => {
                    let values = S::xap_opt(x1, x2, i);
                    let result = P::add_thread_optionals(out, values);
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

                match chunk_puller.pull() {
                    Some(chunk) => {
                        let values = chunk.flat_map(|i| S::xap_opt(x1, x2, i));
                        let result = P::add_thread_optionals(out, values);
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
