use crate::{
    ThreadRunner,
    computations::{Using, Values},
};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn u_xfx<C, U, I, Vt, Vo, M1, F, M2>(
    mut runner: C,
    mut u: U::Item,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
    filter: &F,
    map2: &M2,
) -> Option<(usize, Vo::Item)>
where
    C: ThreadRunner,
    U: Using,
    I: ConcurrentIter,
    Vt: Values,
    Vo: Values,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vt,
    F: Fn(&mut U::Item, &Vt::Item) -> bool + Send + Sync,
    M2: Fn(&mut U::Item, Vt::Item) -> Vo + Send + Sync,
{
    let u = &mut u;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    let vt = map1(u, i);
                    if let Some(first) = vt.u_fx_next(u, filter, map2) {
                        iter.skip_to_end();
                        runner.complete_chunk(shared_state, chunk_size);
                        runner.complete_task(shared_state);
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
                            let vt = map1(u, i);
                            if let Some(first) = vt.u_fx_next(u, filter, map2) {
                                iter.skip_to_end();
                                runner.complete_chunk(shared_state, chunk_size);
                                runner.complete_task(shared_state);
                                return Some((idx, first));
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
