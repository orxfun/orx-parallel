use crate::{
    ThreadRunner,
    values::{TransformableValues, WhilstOption},
};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn u_m<C, U, I, O, M1>(
    mut runner: C,
    mut u: U,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
) -> Option<O>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U, I::Item) -> O,
{
    let u = &mut u;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let first = map1(u, i);
                    iter.skip_to_end();
                    runner.complete_chunk(shared_state, chunk_size);
                    runner.complete_task(shared_state);
                    return Some(first);
                }
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(mut chunk) => {
                        if let Some(i) = chunk.next() {
                            let first = map1(u, i);
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            return Some(first);
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

pub fn u_x<C, U, I, Vo, X1>(
    mut runner: C,
    mut u: U,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &X1,
) -> Option<Vo::Item>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    Vo: TransformableValues,
    Vo::Item: Send,
    X1: Fn(&mut U, I::Item) -> Vo,
{
    let u = &mut u;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let vt = xap1(u, i);
                    match vt.first() {
                        WhilstOption::ContinueSome(first) => {
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            return Some(first);
                        }
                        WhilstOption::ContinueNone => continue,
                        WhilstOption::Stop => {
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            return None;
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
                        for i in chunk {
                            let vt = xap1(u, i);
                            match vt.first() {
                                WhilstOption::ContinueSome(first) => {
                                    iter.skip_to_end();
                                    runner.complete_chunk(shared_state, chunk_size);
                                    runner.complete_task(shared_state);
                                    return Some(first);
                                }
                                WhilstOption::ContinueNone => continue,
                                WhilstOption::Stop => {
                                    iter.skip_to_end();
                                    runner.complete_chunk(shared_state, chunk_size);
                                    runner.complete_task(shared_state);
                                    return None;
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
