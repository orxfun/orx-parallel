use crate::{
    ThreadRunner,
    generic_values::{
        Values,
        runner_results::{Reduce, StopReduce},
    },
};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

// m

pub fn m<U, C, I, O, M1, Red>(
    mut u: U,
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    reduce: &Red,
) -> Option<O>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    M1: Fn(&mut U, I::Item) -> O,
    Red: Fn(&mut U, O, O) -> O,
{
    let u = &mut u;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    let mut acc = None;
    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let y = map1(u, i);
                    acc = match acc {
                        Some(x) => Some(reduce(u, x, y)),
                        None => Some(y),
                    };
                }
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => {
                        let mut res = None;
                        for x in chunk {
                            let b = map1(u, x);
                            res = match res {
                                Some(a) => Some(reduce(u, a, b)),
                                None => Some(b),
                            }
                        }
                        acc = match acc {
                            Some(x) => match res {
                                Some(y) => Some(reduce(u, x, y)),
                                None => Some(x),
                            },
                            None => res,
                        };
                    }
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);
    acc
}

// x

pub fn x<U, C, I, Vo, X1, Red>(
    mut u: U,
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &X1,
    reduce: &Red,
) -> Reduce<Vo>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    Vo: Values,
    X1: Fn(&mut U, I::Item) -> Vo,
    Red: Fn(&mut U, Vo::Item, Vo::Item) -> Vo::Item,
{
    let u = &mut u;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    let mut acc = None;
    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let vo = xap1(u, i);
                    let reduce = vo.u_acc_reduce(u, acc, reduce);
                    acc = match Vo::reduce_to_stop(reduce) {
                        Ok(acc) => acc,
                        Err(stop) => {
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            match stop {
                                StopReduce::DueToWhile { acc } => {
                                    return Reduce::StoppedByWhileCondition { acc };
                                }
                                StopReduce::DueToError { error } => {
                                    return Reduce::StoppedByError { error };
                                }
                            }
                        }
                    };
                }
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => {
                        for i in chunk {
                            let vo = xap1(u, i);
                            let reduce = vo.u_acc_reduce(u, acc, reduce);
                            acc = match Vo::reduce_to_stop(reduce) {
                                Ok(acc) => acc,
                                Err(stop) => {
                                    iter.skip_to_end();
                                    runner.complete_chunk(shared_state, chunk_size);
                                    runner.complete_task(shared_state);
                                    match stop {
                                        StopReduce::DueToWhile { acc } => {
                                            return Reduce::StoppedByWhileCondition { acc };
                                        }
                                        StopReduce::DueToError { error } => {
                                            return Reduce::StoppedByError { error };
                                        }
                                    }
                                }
                            };
                        }
                    }
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);

    Reduce::Done { acc }
}
