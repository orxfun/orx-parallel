use crate::{ThreadRunner, values::Values};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

// m

pub fn u_m<C, U, I, O, M1, Red>(
    mut runner: C,
    mut u: U,
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
                    Some(mut chunk) => {
                        acc = match acc {
                            Some(mut acc) => {
                                for a in chunk {
                                    let a = map1(u, a);
                                    acc = reduce(u, acc, a);
                                }
                                Some(acc)
                            }
                            None => match chunk.next() {
                                Some(a) => {
                                    let mut acc = map1(u, a);
                                    for a in chunk {
                                        let a = map1(u, a);
                                        acc = reduce(u, acc, a);
                                    }
                                    Some(acc)
                                }
                                None => None,
                            },
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

pub fn u_x<C, U, I, Vo, X1, Red>(
    mut runner: C,
    mut u: U,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &X1,
    reduce: &Red,
) -> Option<Vo::Item>
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
                    let vo = map1(u, i);
                    acc = vo.u_acc_reduce(u, acc, reduce);
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
                            let vo = map1(u, i);
                            acc = vo.u_acc_reduce(u, acc, reduce);
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
