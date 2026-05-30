use crate::infallible_use::XapUse;
use crate::runner::ParRunner;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn reduce<Q, U, I, X, F>(
    u: &mut U,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x: X,
    f: F,
) -> Option<X::O>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: XapUse<U = U, I = I::Item>,
    F: Fn(&mut X::U, X::O, X::O) -> X::O,
{
    let mut chunk_puller = iter.chunk_puller_by(0, th_idx);
    let u = u as *mut U;

    let mut acc = None;

    // discover first aggregate
    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => {
                match iter.next_by(th_idx) {
                    Some(i) => {
                        let result = x
                            .xap_use(u, i)
                            .into_iter()
                            .reduce(|a, b| f(unsafe { &mut *u }, a, b));
                        if result.is_some() {
                            acc = result;
                            break;
                        }
                    }
                    // TODO: a good back-off strategy might be used, needs benchmark with ConcurrentQueue
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
            c => {
                chunk_puller.resize_for_chunk_size(c);

                match chunk_puller.pull() {
                    Some(chunk) => {
                        let result = chunk
                            .flat_map(|i| x.xap_use(u, i))
                            .reduce(|a, b| f(unsafe { &mut *u }, a, b));
                        if result.is_some() {
                            acc = result;
                            break;
                        }
                    }
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }
        Q::complete_chunk(state, chunk_state);
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
                        match iter.next_by(th_idx) {
                            Some(i) => {
                                let result = x
                                    .xap_use(u, i)
                                    .into_iter()
                                    .reduce(|a, b| f(unsafe { &mut *u }, a, b));
                                if let Some(y) = result {
                                    acc = f(unsafe { &mut *u }, acc, y);
                                }
                            }
                            // TODO: a good back-off strategy might be used, needs benchmark with ConcurrentQueue
                            None if iter.is_completed_when_none_returned() => break,
                            None => {}
                        }
                    }
                    c => {
                        chunk_puller.resize_for_chunk_size(c);

                        match chunk_puller.pull() {
                            Some(chunk) => {
                                let result = chunk
                                    .flat_map(|i| x.xap_use(u, i))
                                    .reduce(|a, b| f(unsafe { &mut *u }, a, b));
                                if let Some(y) = result {
                                    acc = f(unsafe { &mut *u }, acc, y);
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

    result
}
