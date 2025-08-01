use crate::runner::thread_runner_compute::ThreadRunnerCompute;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub fn m_collect_ordered<C, I, O, M1, P>(
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    o_bag: &ConcurrentOrderedBag<O, P>,
    offset: usize,
) where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, value)) => unsafe { o_bag.set_value(offset + idx, map1(value)) },
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((begin_idx, chunk)) => {
                        let values = chunk.map(map1);
                        unsafe { o_bag.set_values(offset + begin_idx, values) };
                    }
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);
}

pub fn using_m_collect_ordered<C, U, I, O, M1, P>(
    mut runner: C,
    mut using: U,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    o_bag: &ConcurrentOrderedBag<O, P>,
    offset: usize,
) where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U, I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let u = &mut using;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, value)) => unsafe { o_bag.set_value(offset + idx, map1(u, value)) },
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((begin_idx, chunk)) => {
                        let values = chunk.map(|value| map1(u, value));
                        unsafe { o_bag.set_values(offset + begin_idx, values) };
                    }
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);
}
