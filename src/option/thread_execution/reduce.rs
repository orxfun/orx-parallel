use crate::option::xap_opt::XapOpt;
use crate::runner::ParRunner;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

/// Returns either of the following:
///
/// * Some(Some(aggregate)): no failure, returns the aggregate
/// * Some(None): no failure but also no element to reduce
/// * None: a failure (None) is observed
pub fn reduce<Q, I, X, F>(
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x: X,
    f: F,
) -> Option<Option<X::O>>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: XapOpt<I = I::Item>,
    F: Fn(X::O, X::O) -> X::O,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    let mut acc = None;

    // discover first aggregate
    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => {
                match item_puller.next() {
                    Some(i) => {
                        for a in x.xap_res(i) {
                            acc = match (a, acc.is_some()) {
                                (Some(a), true) => acc.map(|agg| f(agg, a)),
                                (Some(a), false) => Some(a),
                                (None, _) => {
                                    Q::broadcast_stop(iter, state, chunk_state);
                                    return None;
                                }
                            };
                        }
                    }
                    // TODO: a good back-off strategy might be used, needs benchmark with ConcurrentQueue
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => {
                        for a in chunk.flat_map(|i| x.xap_res(i)) {
                            acc = match (a, acc.is_some()) {
                                (Some(a), true) => acc.map(|agg| f(agg, a)),
                                (Some(a), false) => Some(a),
                                (None, _) => {
                                    Q::broadcast_stop(iter, state, chunk_state);
                                    return None;
                                }
                            };
                        }
                    }
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }
        Q::complete_chunk(state, chunk_state);

        if acc.is_some() {
            break;
        }
    }

    // fold over the aggregate
    let result = match acc {
        None => None,
        Some(mut acc) => {
            loop {
                let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
                let chunk_state = Q::begin_chunk(th_idx, chunk_size);

                match chunk_size {
                    0 | 1 => {
                        match item_puller.next() {
                            Some(i) => {
                                for a in x.xap_res(i) {
                                    acc = match a {
                                        Some(a) => f(acc, a),
                                        None => {
                                            Q::broadcast_stop(iter, state, chunk_state);
                                            return None;
                                        }
                                    };
                                }
                            }
                            // TODO: a good back-off strategy might be used, needs benchmark with ConcurrentQueue
                            None if iter.is_completed_when_none_returned() => break,
                            None => {}
                        }
                    }
                    c => {
                        if c > chunk_puller.chunk_size() {
                            chunk_puller = iter.chunk_puller(c);
                        }

                        match chunk_puller.pull() {
                            Some(chunk) => {
                                for a in chunk.flat_map(|i| x.xap_res(i)) {
                                    acc = match a {
                                        Some(a) => f(acc, a),
                                        None => {
                                            Q::broadcast_stop(iter, state, chunk_state);
                                            return None;
                                        }
                                    };
                                }
                            }
                            None if iter.is_completed_when_none_returned() => break,
                            None => {}
                        }
                    }
                }

                Q::complete_chunk(state, chunk_state);
            }

            Some(acc)
        }
    };

    Some(result)
}
