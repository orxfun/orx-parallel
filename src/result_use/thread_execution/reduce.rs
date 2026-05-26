use crate::infallible_use::{Use, XapUse};
use crate::runner::ParRunner;
use crate::sizes::SizePair;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn reduce<Q, U, I, M, E, X1, X2, S, F>(
    _: S,
    u: &U,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x1: X1,
    x2: X2,
    f: F,
) -> Result<Option<X2::O>, E>
where
    Q: ParRunner,
    U: Use,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    F: Fn(&mut U::Item, X2::O, X2::O) -> X2::O,
{
    let mut u = u.create(th_idx);
    let u = &mut u as *mut U::Item;

    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    let mut acc = None;

    // discover first aggregate
    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => {
                match item_puller.next() {
                    Some(i) => {
                        for a in S::xap_use_res(u, x1, x2, i) {
                            acc = match (a, acc.is_some()) {
                                (Ok(a), true) => acc.map(|agg| f(unsafe { &mut *u }, agg, a)),
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
                        for a in chunk.flat_map(|i| S::xap_use_res(u, x1, x2, i)) {
                            acc = match (a, acc.is_some()) {
                                (Ok(a), true) => acc.map(|agg| f(unsafe { &mut *u }, agg, a)),
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
                let chunk_size = Q::next_chunk_size(state, iter.size_hint());
                let chunk_state = Q::begin_chunk(th_idx, chunk_size);

                match chunk_size {
                    0 | 1 => {
                        match item_puller.next() {
                            Some(i) => {
                                for a in S::xap_use_res(u, x1, x2, i) {
                                    acc = match a {
                                        Ok(a) => f(unsafe { &mut *u }, acc, a),
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
                                let u2 = u;
                                for a in chunk.flat_map(|i| S::xap_use_res(u, x1, x2, i)) {
                                    acc = match a {
                                        Ok(a) => f(unsafe { &mut *u2 }, acc, a),
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
