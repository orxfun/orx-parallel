use crate::ParExtendCore;
use crate::infallible::xap::Xap;
use crate::runner::ParRunner;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn collect<Q, I, X, P>(
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x: X,
) -> P::OrderedThreadValues
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    X::O: Send,
    P: ParExtendCore<X::O>,
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
                    let values = x.xap(i);
                    P::add_ordered_thread_values(out, idx, values);
                }
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                chunk_puller.resize_for_chunk_size(c);

                match chunk_puller.pull_with_idx() {
                    Some((idx, chunk)) => {
                        let values = chunk.flat_map(|i| x.xap(i));
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
