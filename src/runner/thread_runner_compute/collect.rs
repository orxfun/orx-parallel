use crate::{computations::Values, runner::thread_runner::ThreadRunnerCompute};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn x_collect_with_idx<C, I, Vo, M1>(
    mut c: C,
    iter: &I,
    shared_state: &C::SharedState,
    mut map1: M1,
) -> Vec<(usize, Vo::Item)>
where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: FnMut(I::Item) -> Vo + Send,
{
    let mut collected = Vec::new();
    let out_vec = &mut collected;

    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = c.next_chunk_size(shared_state, iter);

        c.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    let vo = map1(i);
                    vo.push_to_vec_with_idx(idx, out_vec);
                }
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((chunk_begin_idx, chunk)) => {
                        for i in chunk {
                            let vo = map1(i);
                            vo.push_to_vec_with_idx(chunk_begin_idx, out_vec);
                        }
                    }
                    None => break,
                }
            }
        }

        c.complete_chunk(shared_state, chunk_size);
    }

    c.complete_task(shared_state);
    collected
}

pub fn xfx_collect_with_idx<C, I, Vt, Vo, M1, F, M2>(
    mut c: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    filter: &F,
    map2: &M2,
) -> Vec<(usize, Vo::Item)>
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
    let mut collected = Vec::new();
    let out_vec = &mut collected;

    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = c.next_chunk_size(shared_state, iter);

        c.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((i_idx, i)) => {
                    let vt = map1(i);
                    vt.xfx_collect_heap(i_idx, filter, map2, out_vec);
                }
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((chunk_begin_idx, chunk)) => {
                        for i in chunk {
                            let vt = map1(i);
                            vt.xfx_collect_heap(chunk_begin_idx, filter, map2, out_vec);
                        }
                    }
                    None => break,
                }
            }
        }

        c.complete_chunk(shared_state, chunk_size);
    }

    c.complete_task(shared_state);
    collected
}
