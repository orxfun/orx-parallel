use crate::ThreadExecutor;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Stop, ThreadCollectArbitrary};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

#[cfg(test)]
pub fn m<U, C, I, O, M1, P>(
    mut using: U,
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    bag: &ConcurrentBag<O, P>,
) where
    C: ThreadExecutor,
    I: ConcurrentIter,
    M1: Fn(&mut U, I::Item) -> O,
    P: IntoConcurrentPinnedVec<O>,
    O: Send,
{
    let u = &mut using;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(value) => _ = bag.push(map1(u, value)),
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => _ = bag.extend(chunk.map(|x| map1(u, x))),
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);
}

// x

pub fn x<U, C, I, Vo, X1, P>(
    mut using: U,
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &X1,
    bag: &ConcurrentBag<Vo::Item, P>,
) -> ThreadCollectArbitrary<Vo::Fallibility>
where
    C: ThreadExecutor,
    I: ConcurrentIter,
    Vo: Values,
    X1: Fn(*mut U, I::Item) -> Vo,
    P: IntoConcurrentPinnedVec<Vo::Item>,
    Vo::Item: Send,
{
    let u = &mut using;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(value) => {
                    // TODO: possible to try to get len and bag.extend(values_vt.values()) when available, same holds for chunk below
                    let vo = xap1(u, value);
                    let done = vo.push_to_bag(bag);

                    if let Some(stop) = Vo::arbitrary_push_to_stop(done) {
                        iter.skip_to_end();
                        runner.complete_chunk(shared_state, chunk_size);
                        runner.complete_task(shared_state);
                        match stop {
                            Stop::DueToWhile => {
                                return ThreadCollectArbitrary::StoppedByWhileCondition;
                            }
                            Stop::DueToError { error } => {
                                return ThreadCollectArbitrary::StoppedByError { error };
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

                match chunk_puller.pull() {
                    Some(chunk) => {
                        for value in chunk {
                            let vo = xap1(u, value);
                            let done = vo.push_to_bag(bag);

                            if let Some(stop) = Vo::arbitrary_push_to_stop(done) {
                                iter.skip_to_end();
                                runner.complete_chunk(shared_state, chunk_size);
                                runner.complete_task(shared_state);
                                match stop {
                                    Stop::DueToWhile => {
                                        return ThreadCollectArbitrary::StoppedByWhileCondition;
                                    }
                                    Stop::DueToError { error } => {
                                        return ThreadCollectArbitrary::StoppedByError { error };
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

    ThreadCollectArbitrary::AllCollected
}
