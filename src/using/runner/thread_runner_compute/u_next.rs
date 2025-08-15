use crate::{
    ThreadRunner,
    values::{TransformableValues, WhilstOption, runner_results::NextWithIdx},
};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn u_m<C, U, I, O, M1>(
    mut runner: C,
    mut u: U,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
) -> Option<(usize, O)>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    M1: Fn(&mut U, I::Item) -> O,
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
                    let first = map1(u, i);
                    iter.skip_to_end();
                    runner.complete_chunk(shared_state, chunk_size);
                    runner.complete_task(shared_state);
                    return Some((idx, first));
                }
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((idx, mut chunk)) => {
                        if let Some(i) = chunk.next() {
                            let first = map1(u, i);
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            return Some((idx, first));
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
) -> NextWithIdx<Vo::Item, ()>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    Vo: TransformableValues,
    X1: Fn(&mut U, I::Item) -> Vo,
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
                    let vt = xap1(u, i);
                    match vt.first_to_depracate() {
                        WhilstOption::ContinueSome(first) => {
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            return NextWithIdx::Found { idx, value: first };
                        }
                        WhilstOption::ContinueNone => continue,
                        WhilstOption::Stop => {
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            return NextWithIdx::StoppedByWhileCondition { idx };
                        }
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
                            let vt = xap1(u, i);
                            match vt.first_to_depracate() {
                                WhilstOption::ContinueSome(first) => {
                                    iter.skip_to_end();
                                    runner.complete_chunk(shared_state, chunk_size);
                                    runner.complete_task(shared_state);
                                    return NextWithIdx::Found { idx, value: first };
                                }
                                WhilstOption::ContinueNone => continue,
                                WhilstOption::Stop => {
                                    iter.skip_to_end();
                                    runner.complete_chunk(shared_state, chunk_size);
                                    runner.complete_task(shared_state);
                                    return NextWithIdx::StoppedByWhileCondition { idx };
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
    NextWithIdx::NotFound
}
