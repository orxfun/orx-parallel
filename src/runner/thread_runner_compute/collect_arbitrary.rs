use crate::{computations::Values, runner::thread_runner_compute::ThreadRunnerCompute};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

#[cfg(test)]
pub fn m<C, I, O, M1, P>(
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    bag: &ConcurrentBag<O, P>,
) where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(value) => _ = bag.push(map1(value)),
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => _ = bag.extend(chunk.map(&map1)),
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);
}

#[cfg(test)]
pub fn using_m<C, U, I, O, M1, P>(
    mut runner: C,
    mut using: U,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    bag: &ConcurrentBag<O, P>,
) where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U, I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
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
                    Some(chunk) => _ = bag.extend(chunk.map(|value| map1(u, value))),
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);
}

// x

pub fn x<C, I, Vo, X1, P>(
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &X1,
    bag: &ConcurrentBag<Vo::Item, P>,
) where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    X1: Fn(I::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(value) => {
                    // TODO: possible to try to get len and bag.extend(values_vt.values()) when available, same holds for chunk below
                    let values_vt = xap1(value);
                    for x in values_vt.values() {
                        bag.push(x);
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
                            let values_vt = xap1(value);
                            for x in values_vt.values() {
                                bag.push(x);
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
}

pub fn using_x_collect_in_arbitrary_order<C, U, I, Vo, X1, P>(
    mut runner: C,
    mut using: U,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &X1,
    bag: &ConcurrentBag<Vo::Item, P>,
) where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    X1: Fn(&mut U, I::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(value) => {
                    // TODO: possible to try to get len and bag.extend(values_vt.values()) when available, same holds for chunk below
                    let values_vt = xap1(&mut using, value);
                    for x in values_vt.values() {
                        bag.push(x);
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
                            let values_vt = xap1(&mut using, value);
                            for x in values_vt.values() {
                                bag.push(x);
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
}
