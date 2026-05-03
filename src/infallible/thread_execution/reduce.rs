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
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    let mut acc = None;

    // discover first aggregate
    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);
        let mut non_empty = false;

        match chunk_size {
            0 | 1 => {
                match item_puller.next() {
                    Some(i) => {
                        non_empty = true;
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
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => {
                        non_empty = true;
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
        match non_empty {
            true => Q::complete_chunk_non_empty(state, chunk_state),
            false => Q::complete_chunk_empty(state, chunk_state),
        }
    }

    // fold over the aggregate
    let result = match acc {
        None => None,
        Some(mut acc) => {
            loop {
                let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
                let chunk_state = Q::begin_chunk(th_idx, chunk_size);
                let mut non_empty = false;

                match chunk_size {
                    0 | 1 => {
                        match item_puller.next() {
                            Some(i) => {
                                non_empty = true;
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
                        if c > chunk_puller.chunk_size() {
                            chunk_puller = iter.chunk_puller(c);
                        }

                        match chunk_puller.pull() {
                            Some(chunk) => {
                                non_empty = true;
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

                match non_empty {
                    true => Q::complete_chunk_non_empty(state, chunk_state),
                    false => Q::complete_chunk_empty(state, chunk_state),
                }
            }

            Some(acc)
        }
    };

    result
}
