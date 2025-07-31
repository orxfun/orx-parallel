use crate::{computations::Values, runner::thread_runner::ThreadRunnerCompute};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn xfx_next<C, I, Vt, Vo, M1, F, M2>(
    mut com: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    filter: &F,
    map2: &M2,
) -> Option<(usize, Vo::Item)>
where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    Vt: Values,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = com.next_chunk_size(shared_state, iter);

        com.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    let vt = map1(i);
                    if let Some(first) = vt.fx_next(filter, map2) {
                        iter.skip_to_end();
                        com.complete_chunk(shared_state, chunk_size);
                        com.complete_task(shared_state);
                        return Some((idx, first));
                    }
                }
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((idx, chunk)) => {
                        for i in chunk {
                            let vt = map1(i);
                            if let Some(first) = vt.fx_next(filter, map2) {
                                iter.skip_to_end();
                                com.complete_chunk(shared_state, chunk_size);
                                com.complete_task(shared_state);
                                return Some((idx, first));
                            }
                        }
                    }
                    None => break,
                }
            }
        }

        com.complete_chunk(shared_state, chunk_size);
    }

    com.complete_task(shared_state);
    None
}

pub fn xfx_next_any<C, I, Vt, Vo, M1, F, M2>(
    mut com: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    filter: &F,
    map2: &M2,
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
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = com.next_chunk_size(shared_state, iter);

        com.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let vt = map1(i);
                    let maybe_next = vt.fx_next(filter, map2);
                    if maybe_next.is_some() {
                        iter.skip_to_end();
                        com.complete_chunk(shared_state, chunk_size);
                        com.complete_task(shared_state);
                        return maybe_next;
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
                            let vt = map1(i);
                            let maybe_next = vt.fx_next(filter, map2);
                            if maybe_next.is_some() {
                                iter.skip_to_end();
                                com.complete_chunk(shared_state, chunk_size);
                                com.complete_task(shared_state);
                                return maybe_next;
                            }
                        }
                    }
                    None => break,
                }
            }
        }

        com.complete_chunk(shared_state, chunk_size);
    }

    com.complete_task(shared_state);
    None
}
