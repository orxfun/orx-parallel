use crate::{
    ThreadExecutor,
    generic_values::Values,
    generic_values::runner_results::{Next, NextWithIdx},
};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn m<U, C, I, O, M1>(
    mut using: U,
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
) -> Option<(usize, O)>
where
    C: ThreadExecutor,
    I: ConcurrentIter,
    M1: Fn(&mut U, I::Item) -> O,
{
    let u = &mut using;
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

pub fn x<U, C, I, Vo, X1>(
    mut using: U,
    mut runner: C,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &X1,
) -> NextWithIdx<Vo>
where
    C: ThreadExecutor,
    I: ConcurrentIter,
    Vo: Values,
    X1: Fn(&mut U, I::Item) -> Vo,
{
    let u = &mut using;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    let vt = xap1(u, i);
                    match vt.next() {
                        Next::Done { value } => {
                            if let Some(value) = value {
                                iter.skip_to_end();
                                runner.complete_chunk(shared_state, chunk_size);
                                runner.complete_task(shared_state);
                                return NextWithIdx::Found { idx, value };
                            }
                        }
                        Next::StoppedByError { error } => {
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            return NextWithIdx::StoppedByError { error };
                        }
                        Next::StoppedByWhileCondition => {
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
                            match vt.next() {
                                Next::Done { value } => {
                                    if let Some(value) = value {
                                        iter.skip_to_end();
                                        runner.complete_chunk(shared_state, chunk_size);
                                        runner.complete_task(shared_state);
                                        return NextWithIdx::Found { idx, value };
                                    }
                                }
                                Next::StoppedByError { error } => {
                                    iter.skip_to_end();
                                    runner.complete_chunk(shared_state, chunk_size);
                                    runner.complete_task(shared_state);
                                    return NextWithIdx::StoppedByError { error };
                                }
                                Next::StoppedByWhileCondition => {
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
