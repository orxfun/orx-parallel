use crate::{ThreadRunner, computations::Values};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub fn m<C, I, O, E, M1>(
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    action: impl Fn(usize, O),
) -> Option<(usize, E)>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> Result<O, E>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, input)) => match map1(input) {
                    Ok(value) => action(idx, value),
                    Err(err) => {
                        iter.skip_to_end();
                        runner.complete_chunk(shared_state, chunk_size);
                        runner.complete_task(shared_state);
                        return Some((idx, err));
                    }
                },
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((begin_idx, chunk)) => {
                        for (i, result) in chunk.map(map1).enumerate() {
                            let idx = begin_idx + i;
                            match result {
                                Ok(value) => action(idx, value),
                                Err(err) => {
                                    iter.skip_to_end();
                                    runner.complete_chunk(shared_state, chunk_size);
                                    runner.complete_task(shared_state);
                                    return Some((idx, err));
                                }
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
