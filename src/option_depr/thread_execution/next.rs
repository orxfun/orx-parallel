use crate::{option::xap_opt::XapOpt, results::ValIdx, runner::ParRunner};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

/// Returns either of the following:
///
/// * Some(Some(found)): no failure, found an element
/// * Some(None): no failure but no element to find
/// * None: a failure (None) is observed
pub fn next<Q, I, X>(
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x: X,
) -> Option<Option<ValIdx<X::O>>>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: XapOpt<I = I::Item>,
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
                            Some(a) => return Some(Some(ValIdx::new(a, idx))),
                            None => return None,
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
                                Some(a) => return Some(Some(ValIdx::new(a, idx))),
                                None => return None,
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

    Some(None)
}
