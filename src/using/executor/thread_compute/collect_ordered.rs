use crate::ThreadExecutor;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{StopWithIdx, ThreadCollect};
use alloc::vec::Vec;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub fn m<U, C, I, O, M1, P>(
    mut using: U,
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    o_bag: &ConcurrentOrderedBag<O, P>,
    offset: usize,
) where
    C: ThreadExecutor,
    I: ConcurrentIter,
    M1: Fn(&mut U, I::Item) -> O,
    P: IntoConcurrentPinnedVec<O>,
    O: Send,
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
                        let values = chunk.map(|x| map1(u, x));
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

pub fn x<U, C, I, Vo, X1>(
    mut using: U,
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &X1,
) -> ThreadCollect<Vo>
where
    C: ThreadExecutor,
    I: ConcurrentIter,
    Vo: Values,
    X1: Fn(&mut U, I::Item) -> Vo,
{
    let u = &mut using;
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
                    let vo = xap1(u, i);
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
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((chunk_begin_idx, chunk)) => {
                        for (within_chunk_idx, value) in chunk.enumerate() {
                            let vo = xap1(u, value);
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
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);

    ThreadCollect::AllCollected { vec: collected }
}
