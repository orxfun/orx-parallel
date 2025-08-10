use crate::{ThreadRunner, computations::Values};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

// m

pub fn m<C, I, O, M1, Red>(
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    reduce: &Red,
) -> Option<O>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O,
    Red: Fn(O, O) -> O,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    let mut acc = None;
    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let y = map1(i);
                    acc = match acc {
                        Some(x) => Some(reduce(x, y)),
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
                        let res = chunk.map(map1).reduce(reduce);
                        acc = match acc {
                            Some(x) => match res {
                                Some(y) => Some(reduce(x, y)),
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

pub fn x<C, I, Vo, X1, Red>(
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &X1,
    reduce: &Red,
) -> Option<Vo::Item>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    Vo: Values,
    X1: Fn(I::Item) -> Vo,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    let mut acc = None;
    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let vo = xap1(i);
                    let (stop, vo_acc) = vo.acc_reduce(acc, reduce);
                    acc = vo_acc;

                    if stop {
                        iter.skip_to_end();
                        runner.complete_chunk(shared_state, chunk_size);
                        runner.complete_task(shared_state);
                        return acc;
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
                        for i in chunk {
                            let vo = xap1(i);
                            let (stop, vo_acc) = vo.acc_reduce(acc, reduce);
                            acc = vo_acc;

                            if stop {
                                iter.skip_to_end();
                                runner.complete_chunk(shared_state, chunk_size);
                                runner.complete_task(shared_state);
                                return acc;
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
    acc
}
