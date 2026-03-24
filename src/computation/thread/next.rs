use crate::computation::thread::thread_comp::ThreadComp;
use crate::computation::val_and_idx::ValIdx;
use crate::xap::Xap;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn next<Q, I, X>(exe: &mut Q, state: &Q::SharedState, iter: &I, x: X) -> Option<ValIdx<X::O>>
where
    Q: ThreadComp,
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = exe.next_chunk_size(state, iter);
        exe.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    if let Some(val) = x.xap(i).into_iter().next() {
                        found(exe, state, iter, chunk_size);
                        return Some(ValIdx { val, idx });
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
                        if let Some(val) = chunk.flat_map(|i| x.xap(i).into_iter()).next() {
                            found(exe, state, iter, chunk_size);
                            return Some(ValIdx { val, idx });
                        }
                    }
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }
        exe.complete_chunk(state, chunk_size);
    }

    exe.complete_task(state);
    None
}

fn found<I, Q>(exe: &mut Q, state: &Q::SharedState, iter: &I, chunk_size: usize)
where
    Q: ThreadComp,
    I: ConcurrentIter,
{
    iter.skip_to_end();
    exe.complete_chunk(state, chunk_size);
    exe.complete_task(state);
}
