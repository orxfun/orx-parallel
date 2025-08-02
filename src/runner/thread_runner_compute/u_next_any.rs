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
    xap1: &M1,
    filter: &F,
    xap2: &M2,
) -> Option<Vo::Item>
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
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let vt = xap1(u, i);
                    let maybe_next = vt.u_fx_next(u, filter, xap2);
                    if maybe_next.is_some() {
                        iter.skip_to_end();
                        runner.complete_chunk(shared_state, chunk_size);
                        runner.complete_task(shared_state);
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
                            let vt = xap1(u, i);
                            let maybe_next = vt.u_fx_next(u, filter, xap2);
                            if maybe_next.is_some() {
                                iter.skip_to_end();
                                runner.complete_chunk(shared_state, chunk_size);
                                runner.complete_task(shared_state);
                                return maybe_next;
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
