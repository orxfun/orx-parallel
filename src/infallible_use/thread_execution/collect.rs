use crate::results::ValsAndIdx;
use crate::runner::ParRunner;
use crate::{ParExtend, infallible_use::xap::XapUse};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn collect<Q, U, I, X>(
    u: &mut U,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x: X,
) -> ValsAndIdx<X::O>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: XapUse<U = U, I = I::Item>,
    X::O: Send,
{
    let mut collected = ValsAndIdx::new();
    let out = &mut collected;

    let mut chunk_puller = iter.chunk_puller_by(0, th_idx);

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match iter.next_with_idx_by(th_idx) {
                Some((idx, i)) => out.extend(idx, x.xap_use(u, i)),
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                chunk_puller.resize_for_chunk_size(c);

                match chunk_puller.pull_with_idx() {
                    Some((idx, chunk)) => out.extend(idx, chunk.flat_map(|i| x.xap_use(u, i))),
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }

        Q::complete_chunk(state, chunk_state);
    }

    collected
}

pub fn collect_x<Q, U, I, X, P>(
    u: &mut U,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x: X,
) -> P::OrderedThreadValues
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: XapUse<U = U, I = I::Item>,
    X::O: Send,
    P: ParExtend<X::O>,
{
    let mut collected = P::new_ordered_thread_values();
    let out = &mut collected;

    let mut chunk_puller = iter.chunk_puller_by(0, th_idx);

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match iter.next_with_idx_by(th_idx) {
                Some((idx, i)) => {
                    let values = x.xap_use(u, i);
                    P::add_ordered_thread_values(out, idx, values);
                }
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                chunk_puller.resize_for_chunk_size(c);

                match chunk_puller.pull_with_idx() {
                    Some((idx, chunk)) => {
                        let values = chunk.flat_map(|i| x.xap_use(u, i));
                        P::add_ordered_thread_values(out, idx, values);
                    }
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }

        Q::complete_chunk(state, chunk_state);
    }

    collected
}
