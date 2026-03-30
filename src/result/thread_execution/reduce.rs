use crate::result::xap_res::XapRes;
use crate::runner::ParRunner;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn reduce<Q, I, X, F>(
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x: X,
    f: F,
) -> Result<Option<X::O>, X::E>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: XapRes<I = I::Item>,
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
                                (Ok(a), true) => acc.map(|agg| f(agg, a)),
                                (Ok(a), false) => Some(a),
                                (Err(e), _) => {
                                    Q::broadcast_stop(iter, state, chunk_state);
                                    return Err(e);
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
                                (Ok(a), true) => acc.map(|agg| f(agg, a)),
                                (Ok(a), false) => Some(a),
                                (Err(e), _) => {
                                    Q::broadcast_stop(iter, state, chunk_state);
                                    return Err(e);
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
                                        Ok(a) => f(acc, a),
                                        Err(e) => {
                                            Q::broadcast_stop(iter, state, chunk_state);
                                            return Err(e);
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
                                        Ok(a) => f(acc, a),
                                        Err(e) => {
                                            Q::broadcast_stop(iter, state, chunk_state);
                                            return Err(e);
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

    Ok(result)
}
