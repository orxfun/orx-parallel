use crate::{ThreadRunner, computations::Values};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

pub fn m<C, I, O, M1, S, SW, P>(
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    stop: S,
    o_bag: &ConcurrentOrderedBag<O, P>,
    offset: usize,
) -> Option<(usize, SW)>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O,
    S: Fn(&O) -> Option<SW>,
    P: IntoConcurrentPinnedVec<O>,
    O: Send,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, value)) => {
                    let value = map1(value);
                    match stop(&value) {
                        Some(stop_with) => {
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            return Some((idx, stop_with));
                        }
                        None => unsafe { o_bag.set_value(offset + idx, value) },
                    }
                }
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((begin_idx, chunk)) => {
                        for (i, value) in chunk.enumerate() {
                            let idx = begin_idx + i;
                            let value = map1(value);
                            match stop(&value) {
                                Some(stop_with) => {
                                    iter.skip_to_end();
                                    runner.complete_chunk(shared_state, chunk_size);
                                    runner.complete_task(shared_state);
                                    return Some((idx, stop_with));
                                }
                                None => unsafe { o_bag.set_value(offset + idx, value) },
                            }
                        }
                    }
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);
    None
}
