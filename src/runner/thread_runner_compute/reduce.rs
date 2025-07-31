use crate::{computations::Values, runner::thread_runner::ThreadRunnerCompute};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn x_reduce<C, I, Vo, M1, X>(
    mut c: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    reduce: &X,
) -> Option<Vo::Item>
where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
    X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    let mut acc = None;
    loop {
        let chunk_size = c.next_chunk_size(shared_state, iter);

        c.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let vo = map1(i);
                    acc = vo.acc_reduce(acc, reduce);
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
                            let vo = map1(i);
                            acc = vo.acc_reduce(acc, reduce);
                        }
                    }
                    None => break,
                }
            }
        }

        c.complete_chunk(shared_state, chunk_size);
    }

    c.complete_task(shared_state);
    acc
}

pub fn xfx_reduce<C, I, Vt, Vo, M1, F, M2, X>(
    mut c: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    filter: &F,
    map2: &M2,
    reduce: &X,
) -> Option<Vo::Item>
where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    Vt: Values,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    let mut acc = None;

    loop {
        let chunk_size = c.next_chunk_size(shared_state, iter);

        c.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let vt = map1(i);
                    acc = vt.fx_reduce(acc, filter, map2, reduce);
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
                            let vt = map1(i);
                            acc = vt.fx_reduce(acc, filter, map2, reduce);
                        }
                    }
                    None => break,
                }
            }
        }

        c.complete_chunk(shared_state, chunk_size);
    }

    c.complete_task(shared_state);
    acc
}
