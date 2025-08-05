use crate::{ThreadRunner, computations::Values};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

// m

pub fn u_m<C, U, I, O, M1, P>(
    mut runner: C,
    mut u: U,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    o_bag: &ConcurrentOrderedBag<O, P>,
    offset: usize,
) where
    C: ThreadRunner,
    I: ConcurrentIter,
    M1: Fn(&mut U, I::Item) -> O,
    P: IntoConcurrentPinnedVec<O>,
    O: Send,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, value)) => unsafe { o_bag.set_value(offset + idx, map1(&mut u, value)) },
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((begin_idx, chunk)) => {
                        let values = chunk.map(|value| map1(&mut u, value));
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

// x

pub fn u_x<C, U, I, Vo, X1>(
    mut runner: C,
    mut u: U,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &X1,
) -> Vec<(usize, Vo::Item)>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    Vo: Values,
    X1: Fn(&mut U, I::Item) -> Vo,
{
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
                    let vo = xap1(&mut u, i);
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
                            let vo = xap1(&mut u, i);
                            vo.push_to_vec_with_idx(chunk_begin_idx, out_vec);
                        }
                    }
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);

    collected
}

// xfx

pub fn u_xfx<C, U, I, Vt, Vo, M1, F, M2>(
    mut runner: C,
    mut u: U,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &M1,
    filter: &F,
    xap2: &M2,
) -> Vec<(usize, Vo::Item)>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    Vt: Values,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U, I::Item) -> Vt,
    F: Fn(&mut U, &Vt::Item) -> bool + Send + Sync,
    M2: Fn(&mut U, Vt::Item) -> Vo + Send + Sync,
{
    let u = &mut u;
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
                    vo.u_xfx_collect_heap(u, idx, filter, xap2, out_vec);
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
                            let vo = xap1(u, i);
                            vo.u_xfx_collect_heap(u, chunk_begin_idx, filter, xap2, out_vec);
                        }
                    }
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);

    collected
}
