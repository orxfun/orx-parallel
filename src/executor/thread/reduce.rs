use crate::{executor::thread::thread_executor::ThreadExecutor, xap::Xap};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn reduce<Q, I, X, F>(exe: &mut Q, state: &Q::SharedState, iter: &I, x: X, f: F) -> Option<X::O>
where
    Q: ThreadExecutor,
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    F: Fn(X::O, X::O) -> X::O,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    let mut acc = None;

    // discover first aggregate
    loop {
        let chunk_size = exe.next_chunk_size(state, iter);
        exe.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => {
                match item_puller.next() {
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
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => {
                        let result = chunk.flat_map(|i| x.xap(i).into_iter()).reduce(&f);
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
        exe.complete_chunk(state, chunk_size);
    }

    // fold over the aggregate
    let result = match acc {
        None => None,
        Some(mut acc) => {
            loop {
                let chunk_size = exe.next_chunk_size(state, iter);
                exe.begin_chunk(chunk_size);

                match chunk_size {
                    0 | 1 => {
                        match item_puller.next() {
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
                        if c > chunk_puller.chunk_size() {
                            chunk_puller = iter.chunk_puller(c);
                        }

                        match chunk_puller.pull() {
                            Some(chunk) => {
                                let result = chunk.flat_map(|i| x.xap(i).into_iter()).reduce(&f);
                                if let Some(y) = result {
                                    acc = f(acc, y);
                                }
                            }
                            None if iter.is_completed_when_none_returned() => break,
                            None => {}
                        }
                    }
                }
                exe.complete_chunk(state, chunk_size);
            }

            Some(acc)
        }
    };

    exe.complete_task(state);

    result
}
