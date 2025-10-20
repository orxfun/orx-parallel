use crate::ThreadExecutor;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{StopWithIdx, ThreadCollect};
use alloc::vec::Vec;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub fn m<C, I, O, M1, P>(
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    o_bag: &ConcurrentOrderedBag<O, P>,
    offset: usize,
) where
    C: ThreadExecutor,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O,
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
                Some((idx, value)) => unsafe { o_bag.set_value(offset + idx, map1(value)) },
                None => {
                    if iter.is_completed_when_none_returned() {
                        break;
                    }
                }
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
                    None => {
                        if iter.is_completed_when_none_returned() {
                            break;
                        }
                    }
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);
}

pub fn x<C, I, Vo, X1>(
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &X1,
) -> ThreadCollect<Vo>
where
    C: ThreadExecutor,
    I: ConcurrentIter,
    Vo: Values,
    X1: Fn(I::Item) -> Vo,
{
    let mut collected = Vec::new();
    let out_vec = &mut collected;

    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    let vo = xap1(i);
                    let done = vo.push_to_vec_with_idx(idx, out_vec);
                    if let Some(stop) = Vo::ordered_push_to_stop(done) {
                        iter.skip_to_end();
                        runner.complete_chunk(shared_state, chunk_size);
                        runner.complete_task(shared_state);
                        match stop {
                            StopWithIdx::DueToWhile { idx } => {
                                return ThreadCollect::StoppedByWhileCondition {
                                    vec: collected,
                                    stopped_idx: idx,
                                };
                            }
                            StopWithIdx::DueToError { idx: _, error } => {
                                return ThreadCollect::StoppedByError { error };
                            }
                        }
                    }
                }
                None => {
                    if iter.is_completed_when_none_returned() {
                        break;
                    }
                }
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((chunk_begin_idx, chunk)) => {
                        for (within_chunk_idx, value) in chunk.enumerate() {
                            let vo = xap1(value);
                            let done = vo.push_to_vec_with_idx(chunk_begin_idx, out_vec);
                            if let Some(stop) = Vo::ordered_push_to_stop(done) {
                                iter.skip_to_end();
                                runner.complete_chunk(shared_state, chunk_size);
                                runner.complete_task(shared_state);
                                match stop {
                                    StopWithIdx::DueToWhile { idx } => {
                                        return ThreadCollect::StoppedByWhileCondition {
                                            vec: collected,
                                            stopped_idx: idx + within_chunk_idx,
                                        };
                                    }
                                    StopWithIdx::DueToError { idx: _, error } => {
                                        return ThreadCollect::StoppedByError { error };
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        if iter.is_completed_when_none_returned() {
                            break;
                        }
                    }
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);

    ThreadCollect::AllCollected { vec: collected }
}
