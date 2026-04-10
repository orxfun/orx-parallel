use crate::infallible::xap::Xap;
use crate::runner::ParRunner;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub fn collect_arb_over_bag<Q, I, X, P>(
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x: X,
    bag: &ConcurrentBag<X::O, P>,
) where
    Q: ParRunner,
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    P: IntoConcurrentPinnedVec<X::O>,
    X::O: Send,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    // TODO: possible to try to get len and bag.extend(values_vt.values()) when available, same holds for chunk below
                    for i in x.xap(i).into_iter() {
                        _ = bag.push(i);
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
                        for i in chunk.flat_map(|i| x.xap(i)) {
                            _ = bag.push(i);
                        }
                    }
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }

        Q::complete_chunk(state, chunk_state);
    }
}
