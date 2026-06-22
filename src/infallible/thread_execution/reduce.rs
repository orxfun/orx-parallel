use crate::infallible::xap::Xap;
use crate::runner::ParRunner;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn reduce<Q, I, X, F>(th_idx: usize, state: &Q::State, iter: &I, x: X, f: F) -> Option<X::O>
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    F: Fn(X::O, X::O) -> X::O,
{
    let mut chunk_puller = iter.chunk_puller_by(0, th_idx);

    let mut acc = None;

    // discover first aggregate
    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => {
                match iter.next_by(th_idx) {
                    Some(i) => {
                        let result = x.xap(i).into_iter().reduce(&f);
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
                        let result = chunk.flat_map(|i| x.xap(i)).reduce(&f);
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
    

    match acc {
        None => None,
        Some(mut acc) => {
            loop {
                let chunk_size = Q::next_chunk_size(state, iter.size_hint());
                let chunk_state = Q::begin_chunk(th_idx, chunk_size);

                match chunk_size {
                    0 | 1 => {
                        match iter.next_by(th_idx) {
                            Some(i) => {
                                let result = x.xap(i).into_iter().reduce(&f);
                                if let Some(y) = result {
                                    acc = f(acc, y);
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
                                let result = chunk.flat_map(|i| x.xap(i)).reduce(&f);
                                if let Some(y) = result {
                                    acc = f(acc, y);
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
    }
}
